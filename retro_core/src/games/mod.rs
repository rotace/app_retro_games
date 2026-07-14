//! レトロゲーム集モジュール
//!
//! タイトル画面でゲームを選択し、各ゲーム画面でプレイする。
//! `GameCore` トレイトを実装し、ランナーから `tick_frame` で駆動される。
//!
//! ゲームごとの実装はサブモジュールに分割する:
//! - [`breakout`] — ブロック崩し
//! - [`tetris`] — テトリス
//! - [`self_checkout`] — セルフレジおもちゃ
//! - [`draw`] — 共通描画ヘルパ

mod breakout;
mod draw;
mod eg_target;
mod jp_font;
mod self_checkout;
mod tetris;

use crate::{GameCore, InputState, RenderTarget};

use breakout::BreakoutGame;
use draw::{
    clear, draw_text, draw_text_centered, text_width, COLOR_BG, COLOR_CYAN, COLOR_GRAY,
    COLOR_WHITE, COLOR_YELLOW,
};
use self_checkout::SelfCheckoutGame;
use tetris::TetrisGame;

/// エッジ検出（押し始め）
pub(crate) fn just_pressed(prev: bool, curr: bool) -> bool {
    curr && !prev
}

/// タイトルで選択可能なゲーム
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKind {
    Breakout,
    Tetris,
    SelfCheckout,
}

const MENU_ITEMS: [(GameKind, &str); 3] = [
    (GameKind::Breakout, "BREAKOUT"),
    (GameKind::Tetris, "TETRIS"),
    (GameKind::SelfCheckout, "SELF CHECKOUT"),
];

/// 画面状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    Playing(GameKind),
}

/// レトロゲーム集のエントリポイント
pub struct RetroGames {
    screen: Screen,
    menu_index: usize,
    prev_input: InputState,
    breakout: BreakoutGame,
    tetris: TetrisGame,
    self_checkout: SelfCheckoutGame,
    frame: u64,
}

impl RetroGames {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title,
            menu_index: 0,
            prev_input: InputState::default(),
            breakout: BreakoutGame::new(),
            tetris: TetrisGame::new(),
            self_checkout: SelfCheckoutGame::new(),
            frame: 0,
        }
    }

    fn update_title(&mut self, input: &InputState) {
        if just_pressed(self.prev_input.up, input.up) {
            if self.menu_index == 0 {
                self.menu_index = MENU_ITEMS.len() - 1;
            } else {
                self.menu_index -= 1;
            }
        }
        if just_pressed(self.prev_input.down, input.down) {
            self.menu_index = (self.menu_index + 1) % MENU_ITEMS.len();
        }
        if just_pressed(self.prev_input.action, input.action) {
            let kind = MENU_ITEMS[self.menu_index].0;
            match kind {
                GameKind::Breakout => {
                    self.breakout = BreakoutGame::new();
                    self.screen = Screen::Playing(GameKind::Breakout);
                }
                GameKind::Tetris => {
                    self.tetris = TetrisGame::new();
                    self.screen = Screen::Playing(GameKind::Tetris);
                }
                GameKind::SelfCheckout => {
                    self.self_checkout = SelfCheckoutGame::new();
                    self.screen = Screen::Playing(GameKind::SelfCheckout);
                }
            }
        }
    }

    fn render_title<R: RenderTarget>(&self, target: &mut R) {
        clear(target, COLOR_BG);
        let w = target.width();
        draw_text_centered(target, w / 2, 80, "RETRO GAMES", COLOR_YELLOW, 3);
        draw_text_centered(target, w / 2, 130, "SELECT A GAME", COLOR_GRAY, 2);

        for (i, (_, name)) in MENU_ITEMS.iter().enumerate() {
            let y = 200 + i * 44;
            let selected = i == self.menu_index;
            let color = if selected { COLOR_CYAN } else { COLOR_WHITE };
            if selected {
                draw_text_centered(target, w / 2, y, name, color, 2);
                let tw = text_width(name, 2);
                draw_text(
                    target,
                    w as i32 / 2 - tw / 2 - 20,
                    y as i32,
                    ">",
                    COLOR_CYAN,
                    2,
                );
            } else {
                draw_text_centered(target, w / 2, y, name, color, 2);
            }
        }

        draw_text_centered(
            target,
            w / 2,
            target.height().saturating_sub(60),
            "UP/DOWN: SELECT  ACTION: START",
            COLOR_GRAY,
            1,
        );
    }
}

impl Default for RetroGames {
    fn default() -> Self {
        Self::new()
    }
}

impl GameCore for RetroGames {
    fn update(&mut self, input: &InputState) {
        self.frame = self.frame.wrapping_add(1);
        match self.screen {
            Screen::Title => self.update_title(input),
            Screen::Playing(GameKind::Breakout) => {
                self.breakout.update(input, &self.prev_input);
                if self.breakout.wants_title() {
                    self.screen = Screen::Title;
                }
            }
            Screen::Playing(GameKind::Tetris) => {
                self.tetris.update(input, &self.prev_input);
                if self.tetris.wants_title() {
                    self.screen = Screen::Title;
                }
            }
            Screen::Playing(GameKind::SelfCheckout) => {
                self.self_checkout.update(input, &self.prev_input);
                if self.self_checkout.wants_title() {
                    self.screen = Screen::Title;
                }
            }
        }
        self.prev_input = *input;
    }

    fn render<R: RenderTarget>(&self, target: &mut R) {
        match self.screen {
            Screen::Title => self.render_title(target),
            Screen::Playing(GameKind::Breakout) => self.breakout.render(target),
            Screen::Playing(GameKind::Tetris) => self.tetris.render(target),
            Screen::Playing(GameKind::SelfCheckout) => self.self_checkout.render(target),
        }
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
        fn width(&self) -> usize {
            self.width
        }
        fn height(&self) -> usize {
            self.height
        }
        fn stride(&self) -> usize {
            self.width
        }
        fn buffer_mut(&mut self) -> &mut [u32] {
            &mut self.pixels
        }
    }

    #[test]
    fn title_screen_draws_non_black() {
        let mut games = RetroGames::new();
        let mut target = MockTarget::new(640, 480);
        let input = InputState::default();
        games.update(&input);
        games.render(&mut target);
        assert!(target.buffer_mut().iter().any(|&p| p != 0));
    }

    #[test]
    fn select_breakout_from_title() {
        let mut games = RetroGames::new();
        let mut target = MockTarget::new(640, 480);

        let input = InputState {
            action: true,
            ..Default::default()
        };
        games.update(&input);
        games.render(&mut target);

        assert!(matches!(games.screen, Screen::Playing(GameKind::Breakout)));
        assert!(target.buffer_mut().iter().any(|&p| p != 0));
    }

    #[test]
    fn select_tetris_from_title() {
        let mut games = RetroGames::new();
        let down = InputState {
            down: true,
            ..Default::default()
        };
        games.update(&down);
        games.update(&InputState::default());
        let action = InputState {
            action: true,
            ..Default::default()
        };
        games.update(&action);

        assert!(matches!(games.screen, Screen::Playing(GameKind::Tetris)));
    }

    #[test]
    fn select_self_checkout_from_title() {
        let mut games = RetroGames::new();
        let down = InputState {
            down: true,
            ..Default::default()
        };
        // BREAKOUT -> TETRIS -> SELF CHECKOUT
        games.update(&down);
        games.update(&InputState::default());
        games.update(&down);
        games.update(&InputState::default());
        let action = InputState {
            action: true,
            ..Default::default()
        };
        games.update(&action);

        assert!(matches!(
            games.screen,
            Screen::Playing(GameKind::SelfCheckout)
        ));
    }

    #[test]
    fn game_over_returns_to_title() {
        let mut games = RetroGames::new();
        let action = InputState {
            action: true,
            ..Default::default()
        };
        games.update(&action);
        games.update(&InputState::default());

        games.breakout.game_over = true;
        games.update(&action);

        assert!(matches!(games.screen, Screen::Title));
    }
}
