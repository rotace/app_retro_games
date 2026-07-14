//! Linux コンソール (K_MEDIUMRAW) からのキー入力読取
//!
//! make/break の Linux keycode で押下状態を維持し、`InputState` に変換する。

use libc::{self, c_int, fcntl, termios, F_GETFL, F_SETFL, O_NONBLOCK, TCSAFLUSH};
use retro_core::InputState;
use std::io;
use std::os::fd::RawFd;

/// Linux keycode (`linux/input-event-codes.h`)。K_MEDIUMRAW は AT スキャンではなくこちらを返す。
const KEY_ESC: u8 = 1;
const KEY_Q: u8 = 16;
const KEY_W: u8 = 17;
const KEY_A: u8 = 30;
const KEY_S: u8 = 31;
const KEY_D: u8 = 32;
const KEY_Z: u8 = 44;
const KEY_ENTER: u8 = 28;
const KEY_SPACE: u8 = 57;
const KEY_UP: u8 = 103;
const KEY_LEFT: u8 = 105;
const KEY_RIGHT: u8 = 106;
const KEY_DOWN: u8 = 108;

pub struct Keyboard {
    fd: RawFd,
    saved_termios: Option<termios>,
    input: InputState,
    quit: bool,
}

impl Keyboard {
    /// `fd` を非カノニカル・非ブロッキングにし、入力監視を開始する。
    ///
    /// 呼び出し側で事前にキーボードモードを `K_MEDIUMRAW` に切替えておくこと。
    pub fn new(fd: RawFd) -> io::Result<Self> {
        if fd < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid console fd for keyboard",
            ));
        }

        let saved_termios = unsafe {
            let mut term: termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut term) == 0 {
                let mut raw = term;
                raw.c_lflag &= !(libc::ICANON | libc::ECHO);
                raw.c_cc[libc::VMIN] = 0;
                raw.c_cc[libc::VTIME] = 0;
                if libc::tcsetattr(fd, TCSAFLUSH, &raw) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Some(term)
            } else {
                None
            }
        };

        unsafe {
            let flags = fcntl(fd, F_GETFL);
            if flags < 0 {
                return Err(io::Error::last_os_error());
            }
            if fcntl(fd, F_SETFL, flags | O_NONBLOCK) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(Self {
            fd,
            saved_termios,
            input: InputState::default(),
            quit: false,
        })
    }

    /// 利用可能な keycode をすべて読み取り、押下状態を更新する。
    pub fn poll(&mut self) -> InputState {
        let mut buf = [0u8; 64];
        loop {
            let n =
                unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n <= 0 {
                break;
            }
            for &code in &buf[..n as usize] {
                self.apply_keycode(code);
            }
        }
        self.input
    }

    pub fn wants_quit(&self) -> bool {
        self.quit
    }

    fn apply_keycode(&mut self, code: u8) {
        let pressed = (code & 0x80) == 0;
        let key = code & 0x7F;

        match key {
            KEY_ESC | KEY_Q if pressed => self.quit = true,
            KEY_UP | KEY_W => self.input.up = pressed,
            KEY_DOWN | KEY_S => self.input.down = pressed,
            KEY_LEFT | KEY_A => self.input.left = pressed,
            KEY_RIGHT | KEY_D => self.input.right = pressed,
            KEY_SPACE | KEY_ENTER | KEY_Z => self.input.action = pressed,
            _ => {}
        }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if let Some(ref term) = self.saved_termios {
            unsafe {
                let _ = libc::tcsetattr(self.fd, TCSAFLUSH, term);
            }
        }
    }
}

/// キーボードモード定数 (`linux/kd.h`)
pub const K_MEDIUMRAW: c_int = 0x02;
