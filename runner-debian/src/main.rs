mod framebuffer;

use clap::{Parser, Subcommand};
use std::process;
use thiserror::Error;
use std::io::{self, ErrorKind};
use retro_core::RenderTarget;
use std::os::fd::{AsRawFd, OwnedFd, FromRawFd};
use libc::{self, c_int, ioctl, sigaction, SIGINT, SA_RESTART, sighandler_t};
use std::os::raw::c_long;
use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use scopeguard;

static EXIT_SIGNALED: AtomicBool = AtomicBool::new(false);

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Framebuffer error: {0}")]
    Framebuffer(String),
    #[error("Signal handling error: {0}")]
    SignalError(String),
}

// Linux VT ioctl commands
const VT_ACTIVATE: c_long = 0x5606;
const VT_WAITACTIVE: c_long = 0x5607;
const VT_GETSTATE: c_long = 0x5601;

// Linux Keyboard ioctl commands
const KDGKBMODE: c_long = 0x4B44; // Get keyboard mode
const KDSKBMODE: c_long = 0x4B45; // Set keyboard mode
const K_XLATE: c_int = 0x01;      // Translate mode (Ctrl+C enabled)

// Linux TTY ioctl commands
const TIOCSCTTY: c_long = 0x540E; // Set controlling terminal

// Structure for VT_GETSTATE
#[repr(C)]
#[derive(Debug, Default)]
struct vt_stat {
    v_active: u16, // current vt
    v_signal: u16, // signal to send
    v_state: u16,  // vt bitmask
}

fn open_console_fd() -> Result<OwnedFd, RunnerError> {
    println!("Entering open_console_fd");
    let path = CString::new("/dev/tty0").map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
    if fd < 0 {
        let err = io::Error::last_os_error();
        println!("Failed to open /dev/tty0: {:?}", err);
        Err(err.into())
    } else {
        println!("Successfully opened /dev/tty0 with fd: {}", fd);
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

fn get_current_vt(fd: &OwnedFd) -> Result<c_int, RunnerError> {
    println!("Entering get_current_vt");
    let mut vt_state = vt_stat { v_active: 0, v_signal: 0, v_state: 0 };
    let res = unsafe { ioctl(fd.as_raw_fd(), VT_GETSTATE as _, &mut vt_state) };
    if res < 0 {
        let err = io::Error::last_os_error();
        println!("ioctl(VT_GETSTATE) failed: {:?}", err);
        return Err(err.into());
    }

    println!("VT_GETSTATE reports active VT: {}", vt_state.v_active);

    // Fallback: If detected VT is 0 (invalid/special), default to VT 1
    let vt_num = if vt_state.v_active == 0 {
        eprintln!("Warning: Active VT detected as 0. Fallback to VT 1.");
        1
    } else {
        vt_state.v_active as c_int
    };

    println!("Determined original VT: {}", vt_num);
    Ok(vt_num)
}

fn switch_to_vt(fd: &OwnedFd, vt_num: c_int) -> Result<(), RunnerError> {
    println!("Entering switch_to_vt for VT: {}", vt_num);
    let res = unsafe { ioctl(fd.as_raw_fd(), VT_ACTIVATE as _, vt_num) };
    if res < 0 {
        let err = io::Error::last_os_error();
        println!("ioctl(VT_ACTIVATE) failed: {:?}", err);
        return Err(err.into());
    }
    println!("VT_ACTIVATE sent for VT: {}", vt_num);
    let res = unsafe { ioctl(fd.as_raw_fd(), VT_WAITACTIVE as _, vt_num) };
    if res < 0 {
        let err = io::Error::last_os_error();
        println!("ioctl(VT_WAITACTIVE) failed: {:?}", err);
        return Err(err.into());
    }
    println!("VT switched to: {}", vt_num);
    Ok(())
}

fn restore_vt(fd: &OwnedFd, original_vt: c_int) -> Result<(), RunnerError> {
    println!("Entering restore_vt for original VT: {}", original_vt);
    let res = unsafe { ioctl(fd.as_raw_fd(), VT_ACTIVATE as _, original_vt) };
    if res < 0 {
        let err = io::Error::last_os_error();
        println!("ioctl(VT_ACTIVATE) in restore_vt failed: {:?}", err);
        return Err(err.into());
    }
    println!("VT_ACTIVATE sent for original VT: {}", original_vt);
    let res = unsafe { ioctl(fd.as_raw_fd(), VT_WAITACTIVE as _, original_vt) };
    if res < 0 {
        let err = io::Error::last_os_error();
        println!("ioctl(VT_WAITACTIVE) in restore_vt failed: {:?}", err);
        return Err(err.into());
    }
    println!("Restored to original VT: {}", original_vt);
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Draw Perlin noise to the framebuffer
    DrawNoise {
        #[arg(long, default_value_t = 0)]
        width: u32,

        #[arg(long, default_value_t = 0)]
        height: u32,

        #[arg(long, default_value_t = 0.0)]
        x_offset: f32,

        #[arg(long, default_value_t = 0.0)]
        y_offset: f32,

        #[arg(long, default_value_t = 0.1)]
        scale: f32,
    },
}

fn main() {
    println!("Entering main");
    let cli = Cli::parse();

    let console_fd = open_console_fd().expect("Failed to open console device");
    let original_vt = get_current_vt(&console_fd).expect("Failed to get current VT");

    // Save original keyboard mode
    let mut original_kb_mode: c_int = 0;
    unsafe {
        if ioctl(console_fd.as_raw_fd(), KDGKBMODE as _, &mut original_kb_mode) < 0 {
            eprintln!("Warning: Failed to get original keyboard mode");
        } else {
            println!("Original keyboard mode: {}", original_kb_mode);
        }
    }

    // Switch to VT 7 before starting drawing
    if let Err(e) = switch_to_vt(&console_fd, 7) {
        eprintln!("Warning: Failed to switch to VT 7: {}. Continuing on current VT.", e);
    } else {
        // Explicitly open /dev/tty7 to set keyboard mode and controlling terminal
        let vt7_path = CString::new("/dev/tty7").expect("Failed to create CString for /dev/tty7");
        let vt7_fd = unsafe { libc::open(vt7_path.as_ptr(), libc::O_RDWR) };
        
        if vt7_fd >= 0 {
            unsafe {
                // 1. Create a new session to allow changing the controlling terminal
                if libc::setsid() < 0 {
                    eprintln!("Warning: Failed to call setsid()");
                } else {
                    println!("Successfully created new session with setsid()");
                }

                // 2. Set /dev/tty7 as the controlling terminal
                if ioctl(vt7_fd, TIOCSCTTY as _, 1) < 0 {
                    eprintln!("Warning: Failed to set /dev/tty7 as controlling terminal");
                } else {
                    println!("Successfully set /dev/tty7 as controlling terminal");
                }

                // 3. Set keyboard mode to K_XLATE to enable Ctrl+C on VT 7
                if ioctl(vt7_fd, KDSKBMODE as _, K_XLATE) < 0 {
                    eprintln!("Warning: Failed to set keyboard mode to K_XLATE on /dev/tty7");
                } else {
                    println!("Keyboard mode set to K_XLATE on /dev/tty7 successfully");
                }
            }
            
            // Setup SIGINT handler to restore original VT on Ctrl+C
            let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
            sa.sa_flags = 0;
            sa.sa_sigaction = signal_handler as usize;

            unsafe {
                if sigaction(SIGINT, &sa, ptr::null_mut()) != 0 {
                    panic!("Failed to set SIGINT handler");
                }
            }

            let restore_guard = scopeguard::guard((), move |_| {
                println!("Entering restore_guard closure");
                
                // Re-open /dev/tty0 to ensure we have a valid fd for the current session
                let current_console_path = CString::new("/dev/tty0").expect("Failed to create CString for /dev/tty0");
                let current_fd = unsafe { libc::open(current_console_path.as_ptr(), libc::O_RDWR) };
                
                if current_fd >= 0 {
                    let active_fd = unsafe { OwnedFd::from_raw_fd(current_fd) };
                    
                    // 1. Restore original VT
                    println!("Restoring to original VT: {}", original_vt);
                    unsafe {
                        if ioctl(active_fd.as_raw_fd(), VT_ACTIVATE as _, original_vt) < 0 {
                            eprintln!("Error: ioctl(VT_ACTIVATE) failed in guard");
                        }
                        if ioctl(active_fd.as_raw_fd(), VT_WAITACTIVE as _, original_vt) < 0 {
                            eprintln!("Error: ioctl(VT_WAITACTIVE) failed in guard");
                        }
                    }

                    // 2. Restore original keyboard mode
                    unsafe {
                        if ioctl(active_fd.as_raw_fd(), KDSKBMODE as _, original_kb_mode) < 0 {
                            eprintln!("Error: Failed to restore keyboard mode");
                        } else {
                            println!("Keyboard mode restored to: {}", original_kb_mode);
                        }
                    }
                } else {
                    eprintln!("Error: Failed to open /dev/tty0 in restore_guard");
                }
                
                // 3. Finally close VT7 fd
                unsafe {
                    libc::close(vt7_fd);
                    println!("Closed /dev/tty7 fd");
                }
            });

            match cli.command {
                Commands::DrawNoise {
                    width,
                    height,
                    x_offset,
                    y_offset,
                    scale,
                } => {
                    if let Err(e) = handle_draw_noise(width, height, x_offset, y_offset, scale) {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
            }

            // Explicitly drop the guard to ensure restore_vt is called
            drop(restore_guard);
        } else {
            eprintln!("Error: Failed to open /dev/tty7 for keyboard configuration. Ctrl+C may not work.");
            match cli.command {
                Commands::DrawNoise {
                    width,
                    height,
                    x_offset,
                    y_offset,
                    scale,
                } => {
                    if let Err(e) = handle_draw_noise(width, height, x_offset, y_offset, scale) {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        println!("Exiting main normally");
    }
}

// Signal handler function
extern "C" fn signal_handler(_: c_int) {
    EXIT_SIGNALED.store(true, Ordering::SeqCst);
}

fn handle_draw_noise(
    width: u32,
    height: u32,
    x_offset: f32,
    y_offset: f32,
    scale: f32,
) -> Result<(), RunnerError> {
    println!("Entering handle_draw_noise");
    println!("Drawing Perlin noise... (Args: w={}, h={}, x={}, y={}, s={})", width, height, x_offset, y_offset, scale);

    let mut framebuffer = framebuffer::Framebuffer::new("/dev/fb0")?;
    
    let fb_width = framebuffer.width();
    let fb_height = framebuffer.height();
    
    println!("Framebuffer dimensions: {}x{}", fb_width, fb_height);

    let mut noise_gen = retro_core::noise::NoiseGenerator::new(42, scale as f64);
    noise_gen.set_offset(x_offset as f64, y_offset as f64);

    let input = retro_core::InputState::default();
    retro_core::tick_frame(&mut noise_gen, &mut framebuffer, &input);

    println!("Perlin noise drawn successfully to framebuffer! Press Ctrl+C to exit.");

    // Keep the program running until interrupted by a signal
    while !EXIT_SIGNALED.load(Ordering::SeqCst) {
        // Sleep briefly to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Exit signal detected, breaking loop");

    Ok(())
}
