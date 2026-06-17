use core::traits::RenderTarget;
use minifb::{Window, WindowOptions};

pub struct WindowTarget {
    pub window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl WindowTarget {
    pub fn new(width: usize, height: usize) -> Self {
        let window = Window::new(
            "WSLg Noise Drawing - opencode",
            width,
            height,
            WindowOptions::default(),
        ).expect("Failed to create window");

        Self {
            window,
            buffer: vec![0; width * height],
            width,
            height,
        }
    }

    pub fn update(&mut self) {
        self.window.update_with_buffer(&self.buffer, self.width, self.height).expect("Failed to update window");
    }
}

impl RenderTarget for WindowTarget {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
    fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }
    fn buffer(&self) -> &[u32] { &self.buffer }
}
