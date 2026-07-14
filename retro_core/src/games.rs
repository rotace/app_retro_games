//! レトロゲーム集モジュール
//!
//! タイトル画面でゲームを選択し、ブロック崩しまたはテトリスをプレイする。
//! `GameCore` トレイトを実装し、ランナーから `tick_frame` で駆動される。

use crate::{GameCore, InputState, RenderTarget};

// ---------------------------------------------------------------------------
// 色定数 (ARGB8888)
// ---------------------------------------------------------------------------
const COLOR_BG: u32 = 0x0010_1020;
const COLOR_WHITE: u32 = 0x00FF_FFFF;
const COLOR_GRAY: u32 = 0x0088_8888;
const COLOR_YELLOW: u32 = 0x00FF_CC00;
const COLOR_CYAN: u32 = 0x0000_E5E5;
const COLOR_RED: u32 = 0x00E5_3030;
const COLOR_GREEN: u32 = 0x0030_C050;
const COLOR_BLUE: u32 = 0x0030_60E0;
const COLOR_ORANGE: u32 = 0x00E0_8020;
const COLOR_PURPLE: u32 = 0x00A0_40C0;
const COLOR_DARK: u32 = 0x0020_2030;

/// タイトルで選択可能なゲーム
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKind {
    Breakout,
    Tetris,
}

const MENU_ITEMS: [(GameKind, &str); 2] = [
    (GameKind::Breakout, "BREAKOUT"),
    (GameKind::Tetris, "TETRIS"),
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
            frame: 0,
        }
    }

    fn just_pressed(prev: bool, curr: bool) -> bool {
        curr && !prev
    }

    fn update_title(&mut self, input: &InputState) {
        if Self::just_pressed(self.prev_input.up, input.up) {
            if self.menu_index == 0 {
                self.menu_index = MENU_ITEMS.len() - 1;
            } else {
                self.menu_index -= 1;
            }
        }
        if Self::just_pressed(self.prev_input.down, input.down) {
            self.menu_index = (self.menu_index + 1) % MENU_ITEMS.len();
        }
        if Self::just_pressed(self.prev_input.action, input.action) {
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
            }
        }
    }

    fn render_title<R: RenderTarget>(&self, target: &mut R) {
        clear(target, COLOR_BG);
        let w = target.width();
        draw_text_centered(target, w / 2, 80, "RETRO GAMES", COLOR_YELLOW, 3);
        draw_text_centered(target, w / 2, 130, "SELECT A GAME", COLOR_GRAY, 2);

        for (i, (_, name)) in MENU_ITEMS.iter().enumerate() {
            let y = 220 + i * 50;
            let selected = i == self.menu_index;
            let color = if selected { COLOR_CYAN } else { COLOR_WHITE };
            if selected {
                draw_text_centered(target, w / 2, y, name, color, 2);
                // 選択カーソル
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
        }
        self.prev_input = *input;
    }

    fn render<R: RenderTarget>(&self, target: &mut R) {
        match self.screen {
            Screen::Title => self.render_title(target),
            Screen::Playing(GameKind::Breakout) => self.breakout.render(target),
            Screen::Playing(GameKind::Tetris) => self.tetris.render(target),
        }
    }
}

// ---------------------------------------------------------------------------
// 描画ヘルパ
// ---------------------------------------------------------------------------

fn clear<R: RenderTarget>(target: &mut R, color: u32) {
    let buf = target.buffer_mut();
    for p in buf.iter_mut() {
        *p = color;
    }
}

fn put_pixel<R: RenderTarget>(target: &mut R, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 {
        return;
    }
    let x = x as usize;
    let y = y as usize;
    let w = target.width();
    let h = target.height();
    let stride = target.stride();
    if x >= w || y >= h {
        return;
    }
    let buf = target.buffer_mut();
    let idx = y * stride + x;
    if idx < buf.len() {
        buf[idx] = color;
    }
}

fn fill_rect<R: RenderTarget>(target: &mut R, x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 {
        return;
    }
    for dy in 0..h {
        for dx in 0..w {
            put_pixel(target, x + dx, y + dy, color);
        }
    }
}

fn draw_rect<R: RenderTarget>(target: &mut R, x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 {
        return;
    }
    for dx in 0..w {
        put_pixel(target, x + dx, y, color);
        put_pixel(target, x + dx, y + h - 1, color);
    }
    for dy in 0..h {
        put_pixel(target, x, y + dy, color);
        put_pixel(target, x + w - 1, y + dy, color);
    }
}

/// 5x7 簡易ビットマップフォント（ASCII の一部）
fn glyph(c: u8) -> [u8; 7] {
    match c {
        b' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        b'!' => [0x04, 0x04, 0x04, 0x04, 0x00, 0x04, 0x00],
        b'"' => [0x0A, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00],
        b'#' => [0x0A, 0x1F, 0x0A, 0x1F, 0x0A, 0x00, 0x00],
        b'$' => [0x04, 0x0F, 0x14, 0x0E, 0x05, 0x1E, 0x04],
        b'%' => [0x19, 0x19, 0x02, 0x04, 0x08, 0x13, 0x13],
        b'&' => [0x0C, 0x12, 0x14, 0x08, 0x15, 0x12, 0x0D],
        b'\'' => [0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00],
        b'(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        b')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        b'*' => [0x00, 0x0A, 0x04, 0x1F, 0x04, 0x0A, 0x00],
        b'+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        b',' => [0x00, 0x00, 0x00, 0x00, 0x04, 0x04, 0x08],
        b'-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        b'.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00],
        b'/' => [0x01, 0x02, 0x04, 0x08, 0x10, 0x00, 0x00],
        b'0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        b'1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        b'3' => [0x0E, 0x11, 0x01, 0x06, 0x01, 0x11, 0x0E],
        b'4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        b'5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        b'6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        b'7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        b'8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        b'9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        b':' => [0x00, 0x04, 0x00, 0x00, 0x04, 0x00, 0x00],
        b';' => [0x00, 0x04, 0x00, 0x00, 0x04, 0x04, 0x08],
        b'<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02],
        b'=' => [0x00, 0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00],
        b'>' => [0x08, 0x04, 0x02, 0x01, 0x02, 0x04, 0x08],
        b'?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
        b'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        b'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        b'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        b'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        b'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        b'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        b'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        b'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        b'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        b'M' => [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11],
        b'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        b'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        b'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        b'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        b'S' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E],
        b'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        b'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        b'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        b'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        b'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        b'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        b'_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        _ => [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F],
    }
}

fn draw_char<R: RenderTarget>(target: &mut R, x: i32, y: i32, c: u8, color: u32, scale: i32) {
    let g = glyph(c.to_ascii_uppercase());
    let s = scale.max(1);
    for (row, bits) in g.iter().enumerate() {
        for col in 0..5i32 {
            if bits & (1 << (4 - col)) != 0 {
                fill_rect(target, x + col * s, y + row as i32 * s, s, s, color);
            }
        }
    }
}

fn draw_text<R: RenderTarget>(target: &mut R, x: i32, y: i32, text: &str, color: u32, scale: i32) {
    let s = scale.max(1);
    let advance = 6 * s;
    for (i, c) in text.bytes().enumerate() {
        draw_char(target, x + i as i32 * advance, y, c, color, s);
    }
}

fn text_width(text: &str, scale: i32) -> i32 {
    let s = scale.max(1);
    text.len() as i32 * 6 * s
}

fn draw_text_centered<R: RenderTarget>(
    target: &mut R,
    cx: usize,
    y: usize,
    text: &str,
    color: u32,
    scale: i32,
) {
    let tw = text_width(text, scale);
    let x = cx as i32 - tw / 2;
    draw_text(target, x, y as i32, text, color, scale);
}

fn format_u32(buf: &mut [u8], mut n: u32) -> &str {
    if buf.is_empty() {
        return "";
    }
    if n == 0 {
        buf[0] = b'0';
        return std::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut tmp = [0u8; 10];
    let mut len = 0;
    while n > 0 && len < tmp.len() {
        tmp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }
    let out_len = len.min(buf.len());
    for i in 0..out_len {
        buf[i] = tmp[len - 1 - i];
    }
    std::str::from_utf8(&buf[..out_len]).unwrap_or_default()
}

fn draw_score_line<R: RenderTarget>(target: &mut R, x: i32, y: i32, label: &str, value: u32) {
    draw_text(target, x, y, label, COLOR_GRAY, 1);
    let mut num = [0u8; 10];
    let s = format_u32(&mut num, value);
    draw_text(target, x + text_width(label, 1) + 6, y, s, COLOR_WHITE, 1);
}

// ---------------------------------------------------------------------------
// ブロック崩し
// ---------------------------------------------------------------------------

const BO_COLS: usize = 10;
const BO_ROWS: usize = 5;
const BO_BRICK_W: i32 = 52;
const BO_BRICK_H: i32 = 18;
const BO_BRICK_GAP: i32 = 4;
const BO_FIELD_X: i32 = 40;
const BO_FIELD_Y: i32 = 60;
const BO_FIELD_W: i32 = BO_COLS as i32 * (BO_BRICK_W + BO_BRICK_GAP) - BO_BRICK_GAP;
const BO_FIELD_H: i32 = 360;
const BO_PADDLE_W: i32 = 70;
const BO_PADDLE_H: i32 = 12;
const BO_BALL_SIZE: i32 = 8;

struct BreakoutGame {
    bricks: [[bool; BO_COLS]; BO_ROWS],
    paddle_x: i32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    ball_stuck: bool,
    lives: u32,
    score: u32,
    game_over: bool,
    cleared: bool,
    return_to_title: bool,
}

impl BreakoutGame {
    fn new() -> Self {
        let paddle_x = BO_FIELD_X + (BO_FIELD_W - BO_PADDLE_W) / 2;
        let ball_x = paddle_x as f32 + (BO_PADDLE_W as f32 - BO_BALL_SIZE as f32) / 2.0;
        let ball_y = (BO_FIELD_Y + BO_FIELD_H - 40) as f32;
        Self {
            bricks: [[true; BO_COLS]; BO_ROWS],
            paddle_x,
            ball_x,
            ball_y,
            ball_vx: 3.0,
            ball_vy: -3.5,
            ball_stuck: true,
            lives: 3,
            score: 0,
            game_over: false,
            cleared: false,
            return_to_title: false,
        }
    }

    fn wants_title(&self) -> bool {
        self.return_to_title
    }

    fn bricks_remaining(&self) -> usize {
        self.bricks
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&b| b)
            .count()
    }

    fn reset_ball(&mut self) {
        self.ball_stuck = true;
        self.ball_vx = 3.0;
        self.ball_vy = -3.5;
        self.ball_x = self.paddle_x as f32 + (BO_PADDLE_W as f32 - BO_BALL_SIZE as f32) / 2.0;
        self.ball_y = (BO_FIELD_Y + BO_FIELD_H - 40) as f32;
    }

    fn update(&mut self, input: &InputState, prev: &InputState) {
        if self.return_to_title {
            return;
        }
        if self.game_over || self.cleared {
            if RetroGames::just_pressed(prev.action, input.action) {
                self.return_to_title = true;
            }
            return;
        }

        let speed = 6;
        if input.left {
            self.paddle_x = (self.paddle_x - speed).max(BO_FIELD_X);
        }
        if input.right {
            self.paddle_x = (self.paddle_x + speed).min(BO_FIELD_X + BO_FIELD_W - BO_PADDLE_W);
        }

        if self.ball_stuck {
            self.ball_x = self.paddle_x as f32 + (BO_PADDLE_W as f32 - BO_BALL_SIZE as f32) / 2.0;
            self.ball_y = (BO_FIELD_Y + BO_FIELD_H - 40) as f32;
            if RetroGames::just_pressed(prev.action, input.action) {
                self.ball_stuck = false;
            }
            return;
        }

        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;

        // 左右壁
        if self.ball_x <= BO_FIELD_X as f32 {
            self.ball_x = BO_FIELD_X as f32;
            self.ball_vx = self.ball_vx.abs();
        }
        if self.ball_x + BO_BALL_SIZE as f32 >= (BO_FIELD_X + BO_FIELD_W) as f32 {
            self.ball_x = (BO_FIELD_X + BO_FIELD_W - BO_BALL_SIZE) as f32;
            self.ball_vx = -self.ball_vx.abs();
        }
        // 上壁
        if self.ball_y <= BO_FIELD_Y as f32 {
            self.ball_y = BO_FIELD_Y as f32;
            self.ball_vy = self.ball_vy.abs();
        }

        // パドル衝突
        let paddle_y = BO_FIELD_Y + BO_FIELD_H - 28;
        let bx = self.ball_x;
        let by = self.ball_y;
        let bw = BO_BALL_SIZE as f32;
        if by + bw >= paddle_y as f32
            && by + bw <= (paddle_y + BO_PADDLE_H) as f32
            && bx + bw >= self.paddle_x as f32
            && bx <= (self.paddle_x + BO_PADDLE_W) as f32
            && self.ball_vy > 0.0
        {
            self.ball_y = (paddle_y - BO_BALL_SIZE) as f32;
            self.ball_vy = -self.ball_vy.abs();
            // パドル位置による角度調整
            let hit = (bx + bw / 2.0) - (self.paddle_x as f32 + BO_PADDLE_W as f32 / 2.0);
            self.ball_vx = (hit / (BO_PADDLE_W as f32 / 2.0)) * 4.0;
            if self.ball_vx.abs() < 1.5 {
                self.ball_vx = if self.ball_vx < 0.0 { -1.5 } else { 1.5 };
            }
        }

        // 落下
        if self.ball_y > (BO_FIELD_Y + BO_FIELD_H) as f32 {
            if self.lives > 0 {
                self.lives -= 1;
            }
            if self.lives == 0 {
                self.game_over = true;
            } else {
                self.reset_ball();
            }
            return;
        }

        // ブロック衝突
        self.collide_bricks();

        if self.bricks_remaining() == 0 {
            self.cleared = true;
        }
    }

    fn collide_bricks(&mut self) {
        let bx = self.ball_x;
        let by = self.ball_y;
        let bw = BO_BALL_SIZE as f32;
        let origin_x = BO_FIELD_X;
        let origin_y = BO_FIELD_Y + 10;

        for row in 0..BO_ROWS {
            for col in 0..BO_COLS {
                if !self.bricks[row][col] {
                    continue;
                }
                let rx = origin_x + col as i32 * (BO_BRICK_W + BO_BRICK_GAP);
                let ry = origin_y + row as i32 * (BO_BRICK_H + BO_BRICK_GAP);
                let rw = BO_BRICK_W as f32;
                let rh = BO_BRICK_H as f32;
                let rx = rx as f32;
                let ry = ry as f32;

                if bx + bw > rx && bx < rx + rw && by + bw > ry && by < ry + rh {
                    self.bricks[row][col] = false;
                    self.score = self
                        .score
                        .saturating_add(10 * (BO_ROWS as u32 - row as u32));

                    // 衝突面の判定
                    let overlap_l = (bx + bw) - rx;
                    let overlap_r = (rx + rw) - bx;
                    let overlap_t = (by + bw) - ry;
                    let overlap_b = (ry + rh) - by;
                    let min_x = overlap_l.min(overlap_r);
                    let min_y = overlap_t.min(overlap_b);
                    if min_x < min_y {
                        self.ball_vx = -self.ball_vx;
                    } else {
                        self.ball_vy = -self.ball_vy;
                    }
                    return;
                }
            }
        }
    }

    fn render<R: RenderTarget>(&self, target: &mut R) {
        clear(target, COLOR_BG);
        draw_text(target, 40, 16, "BREAKOUT", COLOR_YELLOW, 2);
        draw_score_line(target, 280, 20, "SCORE ", self.score);
        draw_score_line(target, 420, 20, "LIVES ", self.lives);

        draw_rect(
            target,
            BO_FIELD_X - 2,
            BO_FIELD_Y - 2,
            BO_FIELD_W + 4,
            BO_FIELD_H + 4,
            COLOR_GRAY,
        );
        fill_rect(
            target, BO_FIELD_X, BO_FIELD_Y, BO_FIELD_W, BO_FIELD_H, COLOR_DARK,
        );

        let colors = [
            COLOR_RED,
            COLOR_ORANGE,
            COLOR_YELLOW,
            COLOR_GREEN,
            COLOR_CYAN,
        ];
        let origin_x = BO_FIELD_X;
        let origin_y = BO_FIELD_Y + 10;
        for row in 0..BO_ROWS {
            for col in 0..BO_COLS {
                if self.bricks[row][col] {
                    let x = origin_x + col as i32 * (BO_BRICK_W + BO_BRICK_GAP);
                    let y = origin_y + row as i32 * (BO_BRICK_H + BO_BRICK_GAP);
                    fill_rect(
                        target,
                        x,
                        y,
                        BO_BRICK_W,
                        BO_BRICK_H,
                        colors[row % colors.len()],
                    );
                }
            }
        }

        let paddle_y = BO_FIELD_Y + BO_FIELD_H - 28;
        fill_rect(
            target,
            self.paddle_x,
            paddle_y,
            BO_PADDLE_W,
            BO_PADDLE_H,
            COLOR_WHITE,
        );
        fill_rect(
            target,
            self.ball_x as i32,
            self.ball_y as i32,
            BO_BALL_SIZE,
            BO_BALL_SIZE,
            COLOR_CYAN,
        );

        if self.ball_stuck && !self.game_over && !self.cleared {
            draw_text_centered(
                target,
                target.width() / 2,
                (BO_FIELD_Y + BO_FIELD_H / 2) as usize,
                "PRESS ACTION TO LAUNCH",
                COLOR_GRAY,
                1,
            );
        }
        if self.game_over {
            draw_text_centered(
                target,
                target.width() / 2,
                target.height() / 2 - 20,
                "GAME OVER",
                COLOR_RED,
                3,
            );
            draw_text_centered(
                target,
                target.width() / 2,
                target.height() / 2 + 30,
                "ACTION: TITLE",
                COLOR_GRAY,
                1,
            );
        }
        if self.cleared {
            draw_text_centered(
                target,
                target.width() / 2,
                target.height() / 2 - 20,
                "STAGE CLEAR",
                COLOR_GREEN,
                3,
            );
            draw_text_centered(
                target,
                target.width() / 2,
                target.height() / 2 + 30,
                "ACTION: TITLE",
                COLOR_GRAY,
                1,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// テトリス
// ---------------------------------------------------------------------------

const TE_COLS: usize = 10;
const TE_ROWS: usize = 20;
const TE_CELL: i32 = 20;
const TE_OX: i32 = 220;
const TE_OY: i32 = 40;

/// テトロミノ形状 (4回転 × 4セル)
type PieceShape = [[(i8, i8); 4]; 4];

const PIECES: [PieceShape; 7] = [
    // I
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ],
    // O
    [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ],
    // T
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // S
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // Z
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ],
    // J
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    // L
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
];

const PIECE_COLORS: [u32; 7] = [
    COLOR_CYAN,
    COLOR_YELLOW,
    COLOR_PURPLE,
    COLOR_GREEN,
    COLOR_RED,
    COLOR_BLUE,
    COLOR_ORANGE,
];

struct TetrisGame {
    board: [[u8; TE_COLS]; TE_ROWS], // 0=空, 1..=7=ピース色
    piece: usize,
    rotation: usize,
    px: i32,
    py: i32,
    next: usize,
    score: u32,
    lines: u32,
    level: u32,
    drop_timer: u32,
    game_over: bool,
    return_to_title: bool,
    rng: u32,
    das_left: u32,
    das_right: u32,
}

impl TetrisGame {
    fn new() -> Self {
        let mut g = Self {
            board: [[0; TE_COLS]; TE_ROWS],
            piece: 0,
            rotation: 0,
            px: 3,
            py: 0,
            next: 0,
            score: 0,
            lines: 0,
            level: 1,
            drop_timer: 0,
            game_over: false,
            return_to_title: false,
            rng: 0xA5A5_1234,
            das_left: 0,
            das_right: 0,
        };
        g.next = g.rand_piece();
        g.spawn();
        g
    }

    fn wants_title(&self) -> bool {
        self.return_to_title
    }

    fn rand_piece(&mut self) -> usize {
        // xorshift32
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = if x == 0 { 1 } else { x };
        (x % 7) as usize
    }

    fn spawn(&mut self) {
        self.piece = self.next;
        self.next = self.rand_piece();
        self.rotation = 0;
        self.px = 3;
        self.py = 0;
        self.drop_timer = 0;
        if self.collides(self.px, self.py, self.rotation) {
            self.game_over = true;
        }
    }

    fn cells(piece: usize, rotation: usize) -> [(i8, i8); 4] {
        PIECES[piece % 7][rotation % 4]
    }

    fn collides(&self, x: i32, y: i32, rot: usize) -> bool {
        for (dx, dy) in Self::cells(self.piece, rot) {
            let cx = x + dx as i32;
            let cy = y + dy as i32;
            if cx < 0 || cx >= TE_COLS as i32 || cy >= TE_ROWS as i32 {
                return true;
            }
            if cy >= 0 && self.board[cy as usize][cx as usize] != 0 {
                return true;
            }
        }
        false
    }

    fn lock_piece(&mut self) {
        for (dx, dy) in Self::cells(self.piece, self.rotation) {
            let cx = self.px + dx as i32;
            let cy = self.py + dy as i32;
            if cy >= 0 && cy < TE_ROWS as i32 && cx >= 0 && cx < TE_COLS as i32 {
                self.board[cy as usize][cx as usize] = (self.piece as u8) + 1;
            }
        }
        self.clear_lines();
        self.spawn();
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut write_row = TE_ROWS as i32 - 1;
        for read_row in (0..TE_ROWS as i32).rev() {
            let full = self.board[read_row as usize].iter().all(|&c| c != 0);
            if full {
                cleared += 1;
            } else {
                if write_row != read_row {
                    self.board[write_row as usize] = self.board[read_row as usize];
                }
                write_row -= 1;
            }
        }
        while write_row >= 0 {
            self.board[write_row as usize] = [0; TE_COLS];
            write_row -= 1;
        }
        if cleared > 0 {
            self.lines = self.lines.saturating_add(cleared);
            let pts = match cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                _ => 800,
            };
            self.score = self.score.saturating_add(pts * self.level);
            self.level = 1 + self.lines / 10;
        }
    }

    fn drop_interval(&self) -> u32 {
        // フレーム単位（約60FPS想定）
        let base = 48u32.saturating_sub(self.level.saturating_mul(4));
        base.max(8)
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        if !self.collides(self.px + dx, self.py + dy, self.rotation) {
            self.px += dx;
            self.py += dy;
            true
        } else {
            false
        }
    }

    fn try_rotate(&mut self) {
        let next_rot = (self.rotation + 1) % 4;
        // 簡易ウォールキック
        for kick in [0, -1, 1, -2, 2] {
            if !self.collides(self.px + kick, self.py, next_rot) {
                self.px += kick;
                self.rotation = next_rot;
                return;
            }
        }
    }

    fn hard_drop(&mut self) {
        while self.try_move(0, 1) {
            self.score = self.score.saturating_add(2);
        }
        self.lock_piece();
    }

    fn update(&mut self, input: &InputState, prev: &InputState) {
        if self.return_to_title {
            return;
        }
        if self.game_over {
            if RetroGames::just_pressed(prev.action, input.action) {
                self.return_to_title = true;
            }
            return;
        }

        // 横移動（DAS風）
        if input.left {
            let fire = RetroGames::just_pressed(prev.left, input.left) || self.das_left > 10;
            if fire {
                self.try_move(-1, 0);
                if self.das_left > 10 {
                    self.das_left = 8;
                }
            }
            self.das_left = self.das_left.saturating_add(1);
        } else {
            self.das_left = 0;
        }
        if input.right {
            let fire = RetroGames::just_pressed(prev.right, input.right) || self.das_right > 10;
            if fire {
                self.try_move(1, 0);
                if self.das_right > 10 {
                    self.das_right = 8;
                }
            }
            self.das_right = self.das_right.saturating_add(1);
        } else {
            self.das_right = 0;
        }

        if RetroGames::just_pressed(prev.up, input.up) {
            self.try_rotate();
        }
        if RetroGames::just_pressed(prev.action, input.action) {
            self.hard_drop();
            return;
        }

        let soft = if input.down { 2 } else { self.drop_interval() };
        self.drop_timer = self.drop_timer.saturating_add(1);
        if self.drop_timer >= soft {
            self.drop_timer = 0;
            if !self.try_move(0, 1) {
                self.lock_piece();
            } else if input.down {
                self.score = self.score.saturating_add(1);
            }
        }
    }

    fn render<R: RenderTarget>(&self, target: &mut R) {
        clear(target, COLOR_BG);
        draw_text(target, 40, 16, "TETRIS", COLOR_CYAN, 2);
        draw_score_line(target, 40, 60, "SCORE ", self.score);
        draw_score_line(target, 40, 80, "LINES ", self.lines);
        draw_score_line(target, 40, 100, "LEVEL ", self.level);

        draw_text(target, 40, 140, "NEXT", COLOR_GRAY, 1);
        let next_ox = 40;
        let next_oy = 160;
        fill_rect(
            target,
            next_ox,
            next_oy,
            TE_CELL * 5,
            TE_CELL * 4,
            COLOR_DARK,
        );
        for (dx, dy) in Self::cells(self.next, 0) {
            let x = next_ox + (dx as i32 + 1) * TE_CELL;
            let y = next_oy + (dy as i32) * TE_CELL;
            fill_rect(
                target,
                x + 1,
                y + 1,
                TE_CELL - 2,
                TE_CELL - 2,
                PIECE_COLORS[self.next % 7],
            );
        }

        draw_text(target, 450, 60, "CONTROLS", COLOR_GRAY, 1);
        draw_text(target, 450, 80, "LEFT/RIGHT MOVE", COLOR_WHITE, 1);
        draw_text(target, 450, 100, "UP ROTATE", COLOR_WHITE, 1);
        draw_text(target, 450, 120, "DOWN SOFT DROP", COLOR_WHITE, 1);
        draw_text(target, 450, 140, "ACTION HARD DROP", COLOR_WHITE, 1);

        let bw = TE_COLS as i32 * TE_CELL;
        let bh = TE_ROWS as i32 * TE_CELL;
        draw_rect(target, TE_OX - 2, TE_OY - 2, bw + 4, bh + 4, COLOR_GRAY);
        fill_rect(target, TE_OX, TE_OY, bw, bh, COLOR_DARK);

        for row in 0..TE_ROWS {
            for col in 0..TE_COLS {
                let v = self.board[row][col];
                if v != 0 {
                    let color = PIECE_COLORS[(v as usize - 1) % 7];
                    fill_rect(
                        target,
                        TE_OX + col as i32 * TE_CELL + 1,
                        TE_OY + row as i32 * TE_CELL + 1,
                        TE_CELL - 2,
                        TE_CELL - 2,
                        color,
                    );
                }
            }
        }

        if !self.game_over {
            let color = PIECE_COLORS[self.piece % 7];
            for (dx, dy) in Self::cells(self.piece, self.rotation) {
                let cx = self.px + dx as i32;
                let cy = self.py + dy as i32;
                if cy >= 0 {
                    fill_rect(
                        target,
                        TE_OX + cx * TE_CELL + 1,
                        TE_OY + cy * TE_CELL + 1,
                        TE_CELL - 2,
                        TE_CELL - 2,
                        color,
                    );
                }
            }
            // ゴースト
            let mut gy = self.py;
            while !self.collides(self.px, gy + 1, self.rotation) {
                gy += 1;
            }
            if gy != self.py {
                for (dx, dy) in Self::cells(self.piece, self.rotation) {
                    let cx = self.px + dx as i32;
                    let cy = gy + dy as i32;
                    if cy >= 0 {
                        draw_rect(
                            target,
                            TE_OX + cx * TE_CELL + 1,
                            TE_OY + cy * TE_CELL + 1,
                            TE_CELL - 2,
                            TE_CELL - 2,
                            COLOR_GRAY,
                        );
                    }
                }
            }
        }

        if self.game_over {
            draw_text_centered(
                target,
                target.width() / 2,
                target.height() / 2 - 20,
                "GAME OVER",
                COLOR_RED,
                3,
            );
            draw_text_centered(
                target,
                target.width() / 2,
                target.height() / 2 + 30,
                "ACTION: TITLE",
                COLOR_GRAY,
                1,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 単体テスト
// ---------------------------------------------------------------------------

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

        // ACTION でブロック崩し開始（初期選択は BREAKOUT）
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
        // DOWN でテトリスを選択
        let down = InputState {
            down: true,
            ..Default::default()
        };
        games.update(&down);
        // キー離す
        games.update(&InputState::default());
        // ACTION
        let action = InputState {
            action: true,
            ..Default::default()
        };
        games.update(&action);

        assert!(matches!(games.screen, Screen::Playing(GameKind::Tetris)));
    }

    #[test]
    fn breakout_paddle_moves() {
        let mut bo = BreakoutGame::new();
        let start = bo.paddle_x;
        let input = InputState {
            right: true,
            ..Default::default()
        };
        bo.update(&input, &InputState::default());
        assert!(bo.paddle_x > start);
    }

    #[test]
    fn tetris_rotate_changes_rotation() {
        let mut te = TetrisGame::new();
        // O ミノ以外になるまでリロール
        for _ in 0..20 {
            if te.piece != 1 {
                break;
            }
            te.spawn();
        }
        if te.piece == 1 {
            // O のみの場合はスキップ相当
            return;
        }
        let before = te.rotation;
        let up = InputState {
            up: true,
            ..Default::default()
        };
        te.update(&up, &InputState::default());
        assert_ne!(te.rotation, before);
    }

    #[test]
    fn game_over_returns_to_title() {
        let mut games = RetroGames::new();
        let action = InputState {
            action: true,
            ..Default::default()
        };
        games.update(&action); // ブロック崩し開始
        games.update(&InputState::default()); // キー離す

        games.breakout.game_over = true;
        games.update(&action); // ACTION でタイトルへ

        assert!(matches!(games.screen, Screen::Title));
    }
}
