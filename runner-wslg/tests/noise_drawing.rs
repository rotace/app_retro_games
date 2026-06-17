use core::noise::NoiseGenerator;
use core::traits::RenderTarget;
use runner_wslg::WindowTarget;

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

    assert!(target.buffer_mut().iter().any(|&p| p != 0));
}
