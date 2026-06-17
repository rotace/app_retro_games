use core::noise::NoiseGenerator;
use core::traits::RenderTarget;
use minifb::{Window, WindowOptions};

struct WindowTarget {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl WindowTarget {
    fn new(width: usize, height: usize) -> Self {
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

    fn update(&mut self) {
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

#[test]
fn test_noise_drawing() {
    let width = 640;
    let height = 480;
    let mut target = WindowTarget::new(width, height);
    let gen = NoiseGenerator::new(12345, 0.01);

    let mut offset = 0.0;
    for frame in 0..60 {
        gen.render(&mut target, offset).expect("Failed to render noise");
        target.update();
        offset += 0.01;

        if target.window.is_key_down(minifb::Key::Escape) {
            break;
        }
    }

    assert!(target.buffer().iter().any(|&p| p != 0));
}
