//! テトリス

use crate::{InputState, RenderTarget};

use super::draw::{
    clear, draw_rect, draw_score_line, draw_text, draw_text_centered, fill_rect, COLOR_BG,
    COLOR_BLUE, COLOR_CYAN, COLOR_DARK, COLOR_GRAY, COLOR_GREEN, COLOR_ORANGE, COLOR_PURPLE,
    COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};
use super::just_pressed;

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

pub(crate) struct TetrisGame {
    board: [[u8; TE_COLS]; TE_ROWS], // 0=空, 1..=7=ピース色
    pub(crate) piece: usize,
    pub(crate) rotation: usize,
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
    pub(crate) fn new() -> Self {
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

    pub(crate) fn wants_title(&self) -> bool {
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

    pub(crate) fn spawn(&mut self) {
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

    pub(crate) fn update(&mut self, input: &InputState, prev: &InputState) {
        if self.return_to_title {
            return;
        }
        if self.game_over {
            if just_pressed(prev.action, input.action) {
                self.return_to_title = true;
            }
            return;
        }

        // 横移動（DAS風）
        if input.left {
            let fire = just_pressed(prev.left, input.left) || self.das_left > 10;
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
            let fire = just_pressed(prev.right, input.right) || self.das_right > 10;
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

        if just_pressed(prev.up, input.up) {
            self.try_rotate();
        }
        if just_pressed(prev.action, input.action) {
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

    pub(crate) fn render<R: RenderTarget>(&self, target: &mut R) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InputState;

    #[test]
    fn tetris_rotate_changes_rotation() {
        let mut te = TetrisGame::new();
        for _ in 0..20 {
            if te.piece != 1 {
                break;
            }
            te.spawn();
        }
        if te.piece == 1 {
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
}
