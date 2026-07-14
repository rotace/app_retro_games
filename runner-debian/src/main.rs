mod framebuffer;

use clap::{Parser, Subcommand};
use libc::{self, c_int, ioctl};
use std::ffi::CString;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::raw::c_long;
use std::process;
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
const K_XLATE: c_int = 0x01;
const TIOCSCTTY: c_long = 0x540E;

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

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    let cli = Cli::parse();

    // 1. 初期情報の取得と保存
    let console_fd = open_console_fd().expect("Failed to open console device");
    let original_vt = get_current_vt(&console_fd).expect("Failed to get current VT");
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

    // 3. VT 7 の制御奪取とキーボード設定
    let vt7_path = CString::new("/dev/tty7").unwrap();
    let vt7_fd = unsafe { libc::open(vt7_path.as_ptr(), libc::O_RDWR) };
    if vt7_fd >= 0 {
        unsafe {
            libc::setsid();
            let _ = ioctl(vt7_fd, TIOCSCTTY as _, 1);
            let _ = ioctl(vt7_fd, KDSKBMODE as _, K_XLATE);
        }
    } else {
        eprintln!("Warning: Failed to open /dev/tty7. Ctrl+C may not work.");
    }

    // 4. 終了時に必ず元のVTとキーボードを復元するセーフティガード
    let _restore_guard = scopeguard::guard((), move |_| {
        println!("\nRestoring console state...");
        let current_path = CString::new("/dev/tty0").unwrap();
        let current_fd = unsafe { libc::open(current_path.as_ptr(), libc::O_RDWR) };

        if current_fd >= 0 {
            let active_fd = unsafe { OwnedFd::from_raw_fd(current_fd) };
            unsafe {
                // 先にVTを戻す
                let _ = ioctl(active_fd.as_raw_fd(), VT_ACTIVATE as _, original_vt);
                let _ = ioctl(active_fd.as_raw_fd(), VT_WAITACTIVE as _, original_vt);
                // キーボードモードを戻す
                let _ = ioctl(active_fd.as_raw_fd(), KDSKBMODE as _, original_kb_mode);
            }
        }
        if vt7_fd >= 0 {
            unsafe {
                libc::close(vt7_fd);
            }
        }
    });

    // 5. メインロジックの実行
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

fn handle_draw_noise(
    width: u32,
    height: u32,
    x_offset: f32,
    y_offset: f32,
    scale: f32,
) -> Result<(), RunnerError> {
    println!("Drawing Perlin noise... Press Ctrl+C to exit.");

    let mut framebuffer = framebuffer::Framebuffer::new("/dev/fb0")?;
    let mut noise_gen = retro_core::noise::NoiseGenerator::new(42, scale as f64);
    noise_gen.set_offset(x_offset as f64, y_offset as f64);

    let input = retro_core::InputState::default();
    retro_core::tick_frame(&mut noise_gen, &mut framebuffer, &input);

    // Ctrl+C が押されるまでスレッドをブロックし、押されたらループを抜けるチャネルを作成
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .expect("Error setting Ctrl-C handler");

    // シグナルを受信するまで待機（CPU消費ゼロ）
    let _ = rx.recv();
    println!("Exit signal detected.");

    Ok(())
}
