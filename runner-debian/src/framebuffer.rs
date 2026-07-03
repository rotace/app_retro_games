use retro_core::traits::RenderTarget;
use libc::{
    c_int, c_void, mmap, munmap, open, O_RDWR, PROT_READ, PROT_WRITE, MAP_SHARED,
};
use std::{
    ffi::CString,
    io,
    ptr,
};

#[repr(C)]
#[derive(Debug, Default)]
pub struct fb_var_screeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: fb_rgb,
    pub green: fb_rgb,
    pub blue: fb_rgb,
    pub trans: fb_rgb,
    pub nonvisible: u32,
    pub activate: u32,
    pub gamma_blank: u32,
    pub gamma_lut: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct fb_rgb {
    pub r: u32,
    pub g: u32,
    pub b: u32,
}

const FBIOGET_VSCREENINFO: c_int = 0x4600;

pub struct Framebuffer {
    fd: c_int,
    ptr: *mut u32,
    size: usize,
    width: usize,
    height: usize,
}

impl Framebuffer {
    pub fn new(path: &str) -> io::Result<Self> {
        let c_path = CString::new(path).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        let fd = unsafe { open(c_path.as_ptr(), O_RDWR) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut var = fb_var_screeninfo::default();
        let res = unsafe {
            libc::ioctl(fd, FBIOGET_VSCREENINFO as u64, &mut var)
        };

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
                format!("Unsupported bits per pixel: {}. Expected 32 for ARGB8888.", bpp),
            ));
        }

        let size = (width * height * bpp) / 8;

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
        })
    }
}

impl RenderTarget for Framebuffer {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn buffer_mut(&mut self) -> &mut [u32] {
        // SAFETY: The pointer is mapped via mmap and is valid for the lifetime of the Framebuffer.
        // We assume the framebuffer uses 32-bit pixels (ARGB8888) as per the data model.
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.width * self.height)
        }
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
