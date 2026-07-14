use libc::{c_int, c_void, mmap, munmap, open, MAP_SHARED, O_RDWR, PROT_READ, PROT_WRITE};
use retro_core::traits::RenderTarget;
use std::{ffi::CString, io, mem, ptr};

/// `linux/fb.h` の `struct fb_bitfield`
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct fb_bitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

/// `linux/fb.h` の `struct fb_fix_screeninfo`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_fix_screeninfo {
    pub id: [u8; 16],
    /// `unsigned long` — 物理アドレス。ホスト幅は `usize` に合わせる。
    pub smem_start: usize,
    pub smem_len: u32,
    pub type_: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: usize,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

impl Default for fb_fix_screeninfo {
    fn default() -> Self {
        // SAFETY: 全フィールドをゼロ初期化（ioctl 入力として正当）
        unsafe { mem::zeroed() }
    }
}

/// `linux/fb.h` の `struct fb_var_screeninfo`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fb_var_screeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: fb_bitfield,
    pub green: fb_bitfield,
    pub blue: fb_bitfield,
    pub transp: fb_bitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

impl Default for fb_var_screeninfo {
    fn default() -> Self {
        // SAFETY: 全フィールドをゼロ初期化（ioctl 入力として正当）
        unsafe { mem::zeroed() }
    }
}

const FBIOGET_VSCREENINFO: c_int = 0x4600;
/// `linux/fb.h`: `#define FBIOGET_FSCREENINFO 0x4602`（0x4601 は FBIOPUT_VSCREENINFO）
const FBIOGET_FSCREENINFO: c_int = 0x4602;
/// `linux/fb.h`: `#define FBIO_WAITFORVSYNC _IOW('F', 0x20, __u32)` → `0x40044620`
const FBIO_WAITFORVSYNC: c_int = 0x4004_4620;

pub struct Framebuffer {
    fd: c_int,
    /// mmap した実画面バッファ（表示用フロントバッファ）
    ptr: *mut u32,
    size: usize,
    width: usize,
    height: usize,
    stride: usize,
    /// ゲーム描画用のソフトウェア・バックバッファ。
    /// `present()` でフロントへ一括転送し、描画途中のチラつきを防ぐ。
    back: Vec<u32>,
}

impl Framebuffer {
    pub fn new(path: &str) -> io::Result<Self> {
        let c_path =
            CString::new(path).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let fd = unsafe { open(c_path.as_ptr(), O_RDWR) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut var = fb_var_screeninfo::default();
        let res = unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO as _, &mut var) };

        if res < 0 {
            unsafe { libc::close(fd) };
            return Err(io::Error::last_os_error());
        }

        let width = var.xres as usize;
        let height = var.yres as usize;
        let bpp = var.bits_per_pixel as usize;

        if bpp != 32 {
            unsafe { libc::close(fd) };
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported bits per pixel: {}. Expected 32 for ARGB8888.",
                    bpp
                ),
            ));
        }

        let mut fix = fb_fix_screeninfo::default();
        let res_fix = unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO as _, &mut fix) };

        let stride = if res_fix >= 0 && fix.line_length > 0 {
            (fix.line_length as usize) / (bpp / 8)
        } else {
            // For the target device, the actual stride is 5504 bytes / 4 = 1376 px
            // regardless of xres_virtual
            1376
        };

        let size = if res_fix >= 0 && fix.smem_len > 0 {
            fix.smem_len as usize
        } else {
            // Calculate size based on the determined stride
            stride * height * (bpp / 8)
        };

        let frame_pixels = stride.checked_mul(height).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Framebuffer dimensions overflow",
            )
        })?;
        if size < frame_pixels * (bpp / 8) {
            unsafe { libc::close(fd) };
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Framebuffer smem_len ({}) too small for {}x{} stride={}",
                    size, width, height, stride
                ),
            ));
        }

        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            fd,
            ptr: ptr as *mut u32,
            size,
            width,
            height,
            stride,
            back: vec![0; frame_pixels],
        })
    }

    /// 縦ブランキング待ち（未対応ドライバでは無視して続行）
    fn wait_vsync(&self) {
        let mut crtc: u32 = 0;
        // SAFETY: fd は開いたままの framebuffer。引数は有効な u32 ポインタ。
        let _ = unsafe { libc::ioctl(self.fd, FBIO_WAITFORVSYNC as _, &mut crtc) };
    }

    /// バックバッファを実画面へ一括転送する。
    /// `tick_frame` 完了後に呼び、描画途中のフレームが見えないようにする。
    pub fn present(&mut self) {
        self.wait_vsync();

        let frame_pixels = self.stride * self.height;
        // SAFETY: ptr は mmap 済みで Drop まで有効。size は frame_pixels 以上を new() で保証。
        let front = unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size / 4) };
        debug_assert!(front.len() >= frame_pixels);
        debug_assert_eq!(self.back.len(), frame_pixels);
        front[..frame_pixels].copy_from_slice(&self.back);
    }
}

impl RenderTarget for Framebuffer {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn stride(&self) -> usize {
        self.stride
    }

    fn buffer_mut(&mut self) -> &mut [u32] {
        &mut self.back
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr as *mut c_void, self.size);
            libc::close(self.fd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fb_struct_sizes_match_linux_abi() {
        // linux/fb.h 相当（x86_64 では ulong=8 で fix がより大きくなることもあるが、
        // var は常に 160、i686 の fix は 68〜80 付近）。
        assert_eq!(mem::size_of::<fb_var_screeninfo>(), 160);
        assert!(mem::size_of::<fb_fix_screeninfo>() >= 68);
        assert!(mem::size_of::<fb_fix_screeninfo>() <= 80);
    }
}
