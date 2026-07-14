mod framebuffer;
mod keyboard;

use keyboard::{Keyboard, K_MEDIUMRAW};
use libc::{self, c_int, ioctl};
use retro_core::{tick_frame, InputState, RetroGames};
use std::ffi::CString;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::raw::c_long;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Framebuffer error: {0}")]
    Framebuffer(String),
}

// Linux TTY/VT ioctl constants
const VT_ACTIVATE: c_long = 0x5606;
const VT_WAITACTIVE: c_long = 0x5607;
const VT_GETSTATE: c_long = 0x5601;
const KDGKBMODE: c_long = 0x4B44;
const KDSKBMODE: c_long = 0x4B45;
const TIOCSCTTY: c_long = 0x540E;

/// 目標フレーム間隔（約 60 FPS）
const FRAME_DURATION: Duration = Duration::from_micros(16_666);

#[repr(C)]
#[derive(Debug, Default)]
struct vt_stat {
    v_active: u16,
    v_signal: u16,
    v_state: u16,
}

fn open_console_fd() -> Result<OwnedFd, RunnerError> {
    let path =
        CString::new("/dev/tty0").map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
    if fd < 0 {
        Err(io::Error::last_os_error().into())
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

fn get_current_vt(fd: &OwnedFd) -> Result<c_int, RunnerError> {
    let mut vt_state = vt_stat::default();
    let res = unsafe { ioctl(fd.as_raw_fd(), VT_GETSTATE as _, &mut vt_state) };
    if res < 0 {
        return Err(io::Error::last_os_error().into());
    }
    // Fallback: VT 0 が返ってきた場合は安全のために VT 1 とみなす
    let vt_num = if vt_state.v_active == 0 {
        1
    } else {
        vt_state.v_active as c_int
    };
    Ok(vt_num)
}

fn main() {
    // 1. 初期情報の取得と保存
    let console_fd = match open_console_fd() {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("Failed to open console device: {}", e);
            process::exit(1);
        }
    };
    let original_vt = match get_current_vt(&console_fd) {
        Ok(vt) => vt,
        Err(e) => {
            eprintln!("Failed to get current VT: {}", e);
            process::exit(1);
        }
    };
    let mut original_kb_mode: c_int = 0;
    unsafe {
        if ioctl(
            console_fd.as_raw_fd(),
            KDGKBMODE as _,
            &mut original_kb_mode,
        ) < 0
        {
            eprintln!("Warning: Failed to get original keyboard mode");
        }
    }

    // 2. VT 7 への切り替え処理
    unsafe {
        if ioctl(console_fd.as_raw_fd(), VT_ACTIVATE as _, 7) < 0
            || ioctl(console_fd.as_raw_fd(), VT_WAITACTIVE as _, 7) < 0
        {
            eprintln!("Warning: Failed to switch to VT 7. Continuing on current VT.");
        }
    }

    // 3. VT 7 の制御奪取とキーボード設定（ゲーム用に MEDIUMRAW）
    let vt7_path = match CString::new("/dev/tty7") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid path /dev/tty7");
            process::exit(1);
        }
    };
    let vt7_fd = unsafe { libc::open(vt7_path.as_ptr(), libc::O_RDWR) };
    if vt7_fd >= 0 {
        unsafe {
            libc::setsid();
            let _ = ioctl(vt7_fd, TIOCSCTTY as _, 1);
            let _ = ioctl(vt7_fd, KDSKBMODE as _, K_MEDIUMRAW);
        }
    } else {
        eprintln!("Warning: Failed to open /dev/tty7. Keyboard input may not work.");
    }

    // 4. 終了時に必ず元の VT とキーボードを復元するセーフティガード
    let _restore_guard = scopeguard::guard((), move |_| {
        println!("\nRestoring console state...");
        let Ok(current_path) = CString::new("/dev/tty0") else {
            return;
        };
        let current_fd = unsafe { libc::open(current_path.as_ptr(), libc::O_RDWR) };

        if current_fd >= 0 {
            let active_fd = unsafe { OwnedFd::from_raw_fd(current_fd) };
            unsafe {
                let _ = ioctl(active_fd.as_raw_fd(), VT_ACTIVATE as _, original_vt);
                let _ = ioctl(active_fd.as_raw_fd(), VT_WAITACTIVE as _, original_vt);
                let _ = ioctl(active_fd.as_raw_fd(), KDSKBMODE as _, original_kb_mode);
            }
        }
        if vt7_fd >= 0 {
            // VT7 のキーボードモードも復元
            unsafe {
                let _ = ioctl(vt7_fd, KDSKBMODE as _, original_kb_mode);
                libc::close(vt7_fd);
            }
        }
    });

    // 5. メインロジックの実行
    if let Err(e) = run_retro_games(vt7_fd) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_retro_games(input_fd: RawFd) -> Result<(), RunnerError> {
    println!("Starting Retro Games... Esc/Q or Ctrl+C to exit.");

    let mut framebuffer = framebuffer::Framebuffer::new("/dev/fb0")?;
    let mut games = RetroGames::new();

    let mut keyboard = if input_fd >= 0 {
        Some(Keyboard::new(input_fd)?)
    } else {
        None
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_ctrlc = Arc::clone(&running);
    if let Err(e) = ctrlc::set_handler(move || {
        running_ctrlc.store(false, Ordering::SeqCst);
    }) {
        eprintln!("Warning: Failed to set Ctrl-C handler: {}", e);
    }

    while running.load(Ordering::SeqCst) {
        let frame_start = Instant::now();

        let input = if let Some(ref mut kb) = keyboard {
            let input = kb.poll();
            if kb.wants_quit() {
                break;
            }
            input
        } else {
            InputState::default()
        };

        tick_frame(&mut games, &mut framebuffer, &input);

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    println!("Exit signal detected.");
    Ok(())
}
