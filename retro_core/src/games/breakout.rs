//! ブロック崩し

use crate::{InputState, RenderTarget};

use super::draw::{
    clear, draw_rect, draw_score_line, draw_text, draw_text_centered, fill_rect, COLOR_BG,
    COLOR_CYAN, COLOR_DARK, COLOR_GRAY, COLOR_GREEN, COLOR_ORANGE, COLOR_RED, COLOR_WHITE,
    COLOR_YELLOW,
};
use super::just_pressed;

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

pub(crate) struct BreakoutGame {
    bricks: [[bool; BO_COLS]; BO_ROWS],
    pub(crate) paddle_x: i32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    pub(crate) ball_stuck: bool,
    lives: u32,
    score: u32,
    pub(crate) game_over: bool,
    cleared: bool,
    return_to_title: bool,
}

impl BreakoutGame {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn wants_title(&self) -> bool {
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

    pub(crate) fn update(&mut self, input: &InputState, prev: &InputState) {
        if self.return_to_title {
            return;
        }
        if self.game_over || self.cleared {
            if just_pressed(prev.action, input.action) {
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
            if just_pressed(prev.action, input.action) {
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

    pub(crate) fn render<R: RenderTarget>(&self, target: &mut R) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InputState;

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
    fn breakout_launches_ball_on_action() {
        let mut bo = BreakoutGame::new();
        assert!(bo.ball_stuck);
        let action = InputState {
            action: true,
            ..Default::default()
        };
        bo.update(&action, &InputState::default());
        assert!(!bo.ball_stuck);
    }
}
