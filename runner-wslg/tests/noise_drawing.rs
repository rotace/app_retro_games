use core::noise::NoiseGenerator;
use core::{InputState, tick_frame, RenderTarget}; // RenderTarget を追加
use runner_wslg::WindowTarget;

#[test]
fn test_noise_drawing() {
    let width = 640;
    let height = 480;
    let mut target = WindowTarget::new(width, height); // WindowTarget は core::RenderTarget を実装していると仮定
    let mut gen = NoiseGenerator::new(12345, 0.01); // NoiseGenerator を mut に変更

    for frame in 0..60 {
        // InputState を作成し、必要に応じてキー入力をシミュレート
        let mut input = InputState::default();
        // 例: 右キーをシミュレートしてノイズを動かす
        if target.window.is_key_down(minifb::Key::Right) {
             input.right = true;
        }
        // 必要に応じて他のキー入力も追加

        // tick_frame を使用してゲームループを駆動
        tick_frame(&mut gen, &mut target, &input);
        target.update(); // WindowTarget 固有の更新処理

        if target.window.is_key_down(minifb::Key::Escape) {
            break;
        }
    }

    assert!(target.buffer_mut().iter().any(|&p| p != 0));
}
