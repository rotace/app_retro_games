use noise::{NoiseFn, Perlin};
use crate::traits::RenderTarget;
use crate::NoiseError;

pub struct NoiseGenerator {
    perlin: Perlin,
    frequency: f64,
}

impl NoiseGenerator {
    pub fn new(seed: u32, frequency: f64) -> Self {
        Self {
            perlin: Perlin::new(seed),
            frequency,
        }
    }

    pub fn render(&self, target: &mut dyn RenderTarget, offset: f64) -> Result<(), NoiseError> {
        let width = target.width();
        let height = target.height();

        for y in 0..height {
            for x in 0..width {
                let val = self.perlin.get([x as f64 * self.frequency + offset, y as f64 * self.frequency + offset]);
                
                // Map noise value from [-1.0, 1.0] to [0, 255]
                let intensity = ((val + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
                let color = (intensity as u32) << 16 | (intensity as u32) << 8 | (intensity as u32);
                
                target.set_pixel(x, y, color);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTarget {
        pixels: Vec<u32>,
        width: usize,
        height: usize,
    }

    impl MockTarget {
        fn new(width: usize, height: usize) -> Self {
            Self {
                pixels: vec![0; width * height],
                width,
                height,
            }
        }
    }

    impl RenderTarget for MockTarget {
        fn width(&self) -> usize { self.width }
        fn height(&self) -> usize { self.height }
        fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
            self.pixels[y * self.width + x] = color;
        }
        fn buffer(&self) -> &[u32] { &self.pixels }
    }

    #[test]
    fn test_noise_generator_render() {
        let gen = NoiseGenerator::new(1, 0.1);
        let mut target = MockTarget::new(10, 10);
        
        assert!(gen.render(&mut target, 0.0).is_ok());
        assert!(target.buffer().iter().any(|&p| p != 0));
    }
}
