use std::time::Duration;

use minifb::Key;
use retro_core::{tick_frame, InputState, RetroGames};
use runner_wslg::WindowTarget;

fn main() {
    let width = 640;
    let height = 480;
    let mut target = WindowTarget::with_title(width, height, "Retro Games");
    target
        .window
        .limit_update_rate(Some(Duration::from_micros(16_666))); // 約60FPS

    let mut games = RetroGames::new();

    while target.window.is_open() && !target.window.is_key_down(Key::Escape) {
        let input = InputState {
            up: target.window.is_key_down(Key::Up) || target.window.is_key_down(Key::W),
            down: target.window.is_key_down(Key::Down) || target.window.is_key_down(Key::S),
            left: target.window.is_key_down(Key::Left) || target.window.is_key_down(Key::A),
            right: target.window.is_key_down(Key::Right) || target.window.is_key_down(Key::D),
            action: target.window.is_key_down(Key::Space)
                || target.window.is_key_down(Key::Enter)
                || target.window.is_key_down(Key::Z),
        };

        tick_frame(&mut games, &mut target, &input);
        target.update();
    }
}
