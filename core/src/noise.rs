use noise::{NoiseFn, Perlin};
use crate::{InputState, GameCore, RenderTarget};

pub struct NoiseGenerator {
    perlin: Perlin,
    frequency: f64,
    // offset を構造体に持たせる
    offset: f64,
}

impl NoiseGenerator {
    pub fn new(seed: u32, frequency: f64) -> Self {
        Self {
            perlin: Perlin::new(seed),
            frequency,
            offset: 0.0, // 初期オフセット
        }
    }
}

// GameCore トレイトを実装
impl GameCore for NoiseGenerator {
    fn update(&mut self, input: &InputState) {
        // InputState に基づいて offset を更新するロジックを追加
        // 例: 右キーが押されたら offset を増やす
        if input.right {
            self.offset += 0.01; // アニメーション速度の調整
        }
        if input.left {
            self.offset -= 0.01;
        }
        // 他の入力にも対応させる場合はここに追加
    }

    fn render<R: RenderTarget>(&self, target: &mut R) {
        let width = target.width();
        let height = target.height();

        for y in 0..height {
            for x in 0..width {
                // offset を self.offset を使用するように変更
                let val = self.perlin.get([x as f64 * self.frequency + self.offset, y as f64 * self.frequency + self.offset]);

                // Map noise value from [-1.0, 1.0] to [0, 255]
                let intensity = ((val + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
                let color = (intensity as u32) << 16 | (intensity as u32) << 8 | (intensity as u32);

                let buffer = target.buffer_mut();
                buffer[y * width + x] = color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderTarget; // core/src/lib.rs から RenderTarget を使用

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
        fn buffer_mut(&mut self) -> &mut [u32] { &mut self.pixels }
    }

    #[test]
    fn test_noise_generator_gamecore_render() { // テスト名変更
        let mut gen = NoiseGenerator::new(1, 0.1);
        let mut target = MockTarget::new(10, 10);
        let input = InputState::default(); // デフォルトのInputStateを使用

        // updateメソッドも呼び出す
        gen.update(&input);
        // renderメソッドをGameCoreトレイトの定義に従って呼び出す
        gen.render(&mut target); // GameCore トレイトに従った呼び出し

        assert!(target.buffer_mut().iter().any(|&p| p != 0));
    }

    // updateメソッドのテストを追加（任意）
    #[test]
    fn test_noise_generator_update() {
        let mut gen = NoiseGenerator::new(1, 0.1);
        let initial_offset = gen.offset;

        let mut input_right = InputState::default();
        input_right.right = true;
        gen.update(&input_right);
        // オフセットが変化したことを確認 (tolerance for floating point comparisons)
        assert!((gen.offset - (initial_offset + 0.01)).abs() < f64::EPSILON);

        let mut input_left = InputState::default();
        input_left.left = true;
        gen.update(&input_left);
        // オフセットが変化したことを確認
        assert!((gen.offset - (initial_offset + 0.01 - 0.01)).abs() < f64::EPSILON);
    }
}
