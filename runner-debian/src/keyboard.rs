//! Linux コンソール (K_MEDIUMRAW) からのキー入力読取
//!
//! make/break の Linux keycode で押下状態を維持し、`InputState` に変換する。
//! `K_MEDIUMRAW` ではカーネルが Ctrl+Alt+Fn を処理しないため、VT 切替要求もここで検出する。

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
const KEY_KPENTER: u8 = 96;
const KEY_UP: u8 = 103;
const KEY_LEFT: u8 = 105;
const KEY_RIGHT: u8 = 106;
const KEY_DOWN: u8 = 108;
const KEY_LEFTCTRL: u8 = 29;
const KEY_LEFTALT: u8 = 56;
const KEY_RIGHTCTRL: u8 = 97;
const KEY_RIGHTALT: u8 = 100;
const KEY_F1: u8 = 59;
const KEY_F10: u8 = 68;
const KEY_F11: u8 = 87;
const KEY_F12: u8 = 88;

pub struct Keyboard {
    fd: RawFd,
    saved_termios: Option<termios>,
    input: InputState,
    quit: bool,
    ctrl: bool,
    alt: bool,
    /// Ctrl+Alt+Fn で要求された VT 番号（未消費のもの）
    vt_switch_to: Option<c_int>,
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
                // K_MEDIUMRAW のキーコードは tty の「文字」として流れる。
                // ISIG が残っていると、KEY_ENTER(28) == VQUIT(Ctrl-\, 0x1C) により
                // SIGQUIT でプロセスが落ち、キー自体はアプリに届かない。
                raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
                raw.c_iflag &=
                    !(libc::IXON | libc::IXOFF | libc::ICRNL | libc::INLCR | libc::IGNCR);
                raw.c_cc[libc::VMIN] = 0;
                raw.c_cc[libc::VTIME] = 0;
                raw.c_cc[libc::VINTR] = 0;
                raw.c_cc[libc::VQUIT] = 0;
                raw.c_cc[libc::VSUSP] = 0;
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
            ctrl: false,
            alt: false,
            vt_switch_to: None,
        })
    }

    /// 利用可能な keycode をすべて読み取り、押下状態を更新する。
    ///
    /// 同一 `read()` 内で make→break が続く短いタップでも、そのフレームは押下として返す。
    /// （そうしないと `just_pressed` が常に失敗する）
    pub fn poll(&mut self) -> InputState {
        let mut pulsed = InputState::default();
        let mut buf = [0u8; 64];
        loop {
            let n =
                unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n <= 0 {
                break;
            }
            for &code in &buf[..n as usize] {
                let pressed = (code & 0x80) == 0;
                let key = code & 0x7F;
                if pressed {
                    Self::mark_pulse(&mut pulsed, key);
                }
                self.apply_keycode(code);
            }
        }
        InputState {
            up: self.input.up || pulsed.up,
            down: self.input.down || pulsed.down,
            left: self.input.left || pulsed.left,
            right: self.input.right || pulsed.right,
            action: self.input.action || pulsed.action,
        }
    }

    fn mark_pulse(pulsed: &mut InputState, key: u8) {
        match key {
            KEY_UP | KEY_W => pulsed.up = true,
            KEY_DOWN | KEY_S => pulsed.down = true,
            KEY_LEFT | KEY_A => pulsed.left = true,
            KEY_RIGHT | KEY_D => pulsed.right = true,
            KEY_SPACE | KEY_ENTER | KEY_KPENTER | KEY_Z => pulsed.action = true,
            _ => {}
        }
    }

    pub fn wants_quit(&self) -> bool {
        self.quit
    }

    /// Ctrl+Alt+Fn による VT 切替要求を取り出す（あれば）。
    pub fn take_vt_switch(&mut self) -> Option<c_int> {
        self.vt_switch_to.take()
    }

    fn fn_key_to_vt(key: u8) -> Option<c_int> {
        match key {
            KEY_F1..=KEY_F10 => Some((key - KEY_F1 + 1) as c_int),
            KEY_F11 => Some(11),
            KEY_F12 => Some(12),
            _ => None,
        }
    }

    fn apply_keycode(&mut self, code: u8) {
        let pressed = (code & 0x80) == 0;
        let key = code & 0x7F;

        match key {
            KEY_LEFTCTRL | KEY_RIGHTCTRL => self.ctrl = pressed,
            KEY_LEFTALT | KEY_RIGHTALT => self.alt = pressed,
            KEY_ESC | KEY_Q if pressed => self.quit = true,
            KEY_UP | KEY_W => self.input.up = pressed,
            KEY_DOWN | KEY_S => self.input.down = pressed,
            KEY_LEFT | KEY_A => self.input.left = pressed,
            KEY_RIGHT | KEY_D => self.input.right = pressed,
            KEY_SPACE | KEY_ENTER | KEY_KPENTER | KEY_Z => self.input.action = pressed,
            _ => {
                if pressed && self.ctrl && self.alt {
                    if let Some(vt) = Self::fn_key_to_vt(key) {
                        self.vt_switch_to = Some(vt);
                    }
                }
            }
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
