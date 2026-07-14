//! セルフレジを模した幼児向けおもちゃゲーム
//!
//! レイアウト:
//! - 左上: 値段表示テキストボックス
//! - 左下: お野菜ボタン（2×3）
//! - 右上: カート一覧
//! - 右下: ゴウケイボタン
//!
//! 操作: 十字キーでフォーカス移動、ACTION で選択。

use crate::{InputState, RenderTarget};

use super::draw::{
    draw_ellipse, draw_number_7seg, fill_circle, fill_ellipse, fill_rect, format_u32, put_pixel,
    round_rect, COLOR_GREEN, COLOR_ORANGE, COLOR_WHITE,
};
use super::jp_font::{draw_jp_text, draw_jp_text_centered, jp_text_width};
use super::just_pressed;

/// 背景のギンガム（水色チェック）
const COLOR_CHECK_A: u32 = 0x00B8_D4E8;
const COLOR_CHECK_B: u32 = 0x00E8_F4FA;
/// クリーム系ボタン背景
const COLOR_CREAM: u32 = 0x00F8_EDD8;
/// ゴウケイボタンの金色
const COLOR_GOLD: u32 = 0x00E8_C040;
/// 枠線・文字（茶系）
const COLOR_BROWN: u32 = 0x0038_2818;
/// フォーカス強調（枠）
const COLOR_FOCUS: u32 = 0x00E0_5050;
/// フォーカス時のボタン背景
const COLOR_FOCUS_BG: u32 = 0x00FF_F0A0;
const COLOR_GRAY: u32 = 0x0088_8888;

const MAX_CART: usize = 12;
const VEG_COLS: usize = 3;
const VEG_ROWS: usize = 2;
const VEG_COUNT: usize = 6;

/// フォーカス: 0..5 = 野菜, 6 = ゴウケイ, 7 = タイトルへ戻る
const FOCUS_GOUKEI: usize = 6;
const FOCUS_BACK: usize = 7;

#[derive(Debug, Clone, Copy)]
struct Vegetable {
    /// 日本語名（表示用）
    name: &'static str,
    price: u32,
}

const VEGETABLES: [Vegetable; VEG_COUNT] = [
    Vegetable {
        name: "キャベツ",
        price: 150,
    },
    Vegetable {
        name: "トマト",
        price: 100,
    },
    Vegetable {
        name: "にんじん",
        price: 80,
    },
    Vegetable {
        name: "おさかな",
        price: 130,
    },
    Vegetable {
        name: "タマネギ",
        price: 60,
    },
    Vegetable {
        name: "バナナ",
        price: 90,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
    /// 最後に選んだ野菜の値段
    ItemPrice(u32),
    /// ゴウケイで算出した合計
    Total(u32),
    /// 初期状態
    Empty,
}

pub(crate) struct SelfCheckoutGame {
    cart: [Option<usize>; MAX_CART],
    cart_len: usize,
    focus: usize,
    display: DisplayMode,
    return_to_title: bool,
}

impl SelfCheckoutGame {
    pub(crate) fn new() -> Self {
        Self {
            cart: [None; MAX_CART],
            cart_len: 0,
            focus: 0,
            display: DisplayMode::Empty,
            return_to_title: false,
        }
    }

    pub(crate) fn wants_title(&self) -> bool {
        self.return_to_title
    }

    fn cart_total(&self) -> u32 {
        let mut sum = 0u32;
        for i in 0..self.cart_len {
            if let Some(idx) = self.cart[i] {
                if let Some(v) = VEGETABLES.get(idx) {
                    sum = sum.saturating_add(v.price);
                }
            }
        }
        sum
    }

    fn add_to_cart(&mut self, veg_index: usize) {
        if self.cart_len >= MAX_CART {
            return;
        }
        if veg_index >= VEG_COUNT {
            return;
        }
        self.cart[self.cart_len] = Some(veg_index);
        self.cart_len += 1;
        self.display = DisplayMode::ItemPrice(VEGETABLES[veg_index].price);
    }

    fn move_focus(&mut self, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }

        match self.focus {
            0..=5 => {
                let col = (self.focus % VEG_COLS) as i32;
                let row = (self.focus / VEG_COLS) as i32;
                let new_col = col + dx;
                let new_row = row + dy;

                if new_col < 0 || new_col >= VEG_COLS as i32 {
                    // 左右端から右カラム（ゴウケイ / 戻る）へ
                    self.focus = if row == 0 { FOCUS_GOUKEI } else { FOCUS_BACK };
                } else if new_row >= VEG_ROWS as i32 {
                    self.focus = FOCUS_BACK;
                } else if new_row >= 0 {
                    self.focus = (new_row as usize) * VEG_COLS + (new_col as usize);
                }
            }
            FOCUS_GOUKEI => {
                if dx < 0 {
                    self.focus = 2;
                } else if dy > 0 {
                    self.focus = FOCUS_BACK;
                }
            }
            FOCUS_BACK => {
                if dy < 0 {
                    self.focus = FOCUS_GOUKEI;
                } else if dx < 0 {
                    self.focus = 5;
                }
            }
            _ => self.focus = 0,
        }
    }

    pub(crate) fn update(&mut self, input: &InputState, prev: &InputState) {
        if self.return_to_title {
            return;
        }

        if just_pressed(prev.left, input.left) {
            self.move_focus(-1, 0);
        }
        if just_pressed(prev.right, input.right) {
            self.move_focus(1, 0);
        }
        if just_pressed(prev.up, input.up) {
            self.move_focus(0, -1);
        }
        if just_pressed(prev.down, input.down) {
            self.move_focus(0, 1);
        }

        if just_pressed(prev.action, input.action) {
            match self.focus {
                0..=5 => self.add_to_cart(self.focus),
                FOCUS_GOUKEI => {
                    self.display = DisplayMode::Total(self.cart_total());
                }
                FOCUS_BACK => {
                    self.return_to_title = true;
                }
                _ => {}
            }
        }
    }

    pub(crate) fn render<R: RenderTarget>(&self, target: &mut R) {
        draw_gingham(target);

        let w = target.width() as i32;
        let h = target.height() as i32;
        let radius = 12;

        // --- 左上: 値段表示 ---
        let disp_x = 16;
        let disp_y = 14;
        let disp_w = w * 55 / 100 - 24;
        let disp_h = 58;
        round_rect(
            target,
            disp_x,
            disp_y,
            disp_w,
            disp_h,
            radius,
            COLOR_WHITE,
            COLOR_BROWN,
            1,
        );

        draw_jp_text(
            target,
            disp_x + 10,
            disp_y + 20,
            "選んだお野菜の合計：",
            COLOR_BROWN,
            1,
        );
        match self.display {
            DisplayMode::Empty => {
                let label = "---";
                let tw = jp_text_width(label, 1) + jp_text_width("円", 1) + 8;
                let px = disp_x + disp_w - tw - 14;
                draw_jp_text(target, px, disp_y + 22, label, COLOR_GRAY, 1);
                draw_jp_text(
                    target,
                    px + jp_text_width(label, 1) + 6,
                    disp_y + 18,
                    "円",
                    COLOR_GRAY,
                    1,
                );
            }
            DisplayMode::ItemPrice(price) | DisplayMode::Total(price) => {
                let yen_w = jp_text_width("円", 1);
                // 桁数に応じて右寄せ
                let mut tmp = price;
                let mut digits = 1i32;
                while tmp >= 10 {
                    tmp /= 10;
                    digits += 1;
                }
                let num_w = digits * 8 * 2; // scale 2 の 7seg
                let px = disp_x + disp_w - num_w - yen_w - 16;
                let nw = draw_number_7seg(target, px, disp_y + 12, price, COLOR_BROWN, 2);
                draw_jp_text(target, px + nw + 4, disp_y + 18, "円", COLOR_BROWN, 1);
            }
        }

        // --- 左下: お野菜ボタン ---
        let grid_x = 16;
        let grid_y = disp_y + disp_h + 12;
        let grid_w = disp_w;
        let grid_h = h - grid_y - 36;
        let cell_w = (grid_w - 12) / VEG_COLS as i32;
        let cell_h = (grid_h - 8) / VEG_ROWS as i32;

        for (i, veg) in VEGETABLES.iter().enumerate() {
            let col = (i % VEG_COLS) as i32;
            let row = (i / VEG_COLS) as i32;
            let bx = grid_x + 4 + col * cell_w;
            let by = grid_y + 4 + row * cell_h;
            let bw = cell_w - 8;
            let bh = cell_h - 8;
            let focused = self.focus == i;

            let bg = if focused { COLOR_FOCUS_BG } else { COLOR_CREAM };
            let border = if focused { COLOR_FOCUS } else { COLOR_BROWN };
            let thick = if focused { 3 } else { 1 };
            round_rect(target, bx, by, bw, bh, 10, bg, border, thick);

            draw_veggie_icon(target, bx + bw / 2, by + bh / 2 - 18, i, 1);

            draw_jp_text_centered(target, bx + bw / 2, by + bh - 34, veg.name, COLOR_BROWN, 1);

            // 値段（数字 + 円）
            let mut pbuf = [0u8; 10];
            let ps = format_u32(&mut pbuf, veg.price);
            let price_label_w = jp_text_width(ps, 1) + jp_text_width("円", 1);
            let px = bx + bw / 2 - price_label_w / 2;
            draw_jp_text(target, px, by + bh - 20, ps, COLOR_BROWN, 1);
            draw_jp_text(
                target,
                px + jp_text_width(ps, 1),
                by + bh - 20,
                "円",
                COLOR_BROWN,
                1,
            );
        }

        // --- 右上: カート一覧 ---
        let cart_x = w * 55 / 100 + 8;
        let cart_y = 14;
        let cart_w = w - cart_x - 16;
        let cart_h = h - 118;
        round_rect(
            target,
            cart_x,
            cart_y,
            cart_w,
            cart_h,
            radius,
            COLOR_CREAM,
            COLOR_BROWN,
            1,
        );

        draw_jp_text_centered(
            target,
            cart_x + cart_w / 2,
            cart_y + 12,
            "カート一覧",
            COLOR_BROWN,
            1,
        );

        let list_top = cart_y + 40;
        let line_h = 30;
        let visible = ((cart_h - 50) / line_h).max(1) as usize;
        let start = self.cart_len.saturating_sub(visible);
        for (row, ci) in (start..self.cart_len).enumerate() {
            if let Some(idx) = self.cart[ci] {
                let veg = &VEGETABLES[idx];
                let ly = list_top + row as i32 * line_h;
                draw_veggie_icon(target, cart_x + 20, ly + 12, idx, 0);
                draw_jp_text(target, cart_x + 38, ly + 4, veg.name, COLOR_BROWN, 1);

                let mut pbuf = [0u8; 10];
                let ps = format_u32(&mut pbuf, veg.price);
                let tw = jp_text_width(ps, 1) + jp_text_width("円", 1);
                let px = cart_x + cart_w - tw - 12;
                draw_jp_text(target, px, ly + 4, ps, COLOR_BROWN, 1);
                draw_jp_text(
                    target,
                    px + jp_text_width(ps, 1),
                    ly + 4,
                    "円",
                    COLOR_BROWN,
                    1,
                );
            }
        }

        // --- 右下: ゴウケイボタン ---
        let btn_x = cart_x;
        let btn_y = cart_y + cart_h + 10;
        let btn_w = cart_w;
        let btn_h = 52;
        let goukei_focus = self.focus == FOCUS_GOUKEI;
        let gbg = if goukei_focus {
            COLOR_FOCUS_BG
        } else {
            COLOR_GOLD
        };
        let gborder = if goukei_focus {
            COLOR_FOCUS
        } else {
            COLOR_BROWN
        };
        let thick = if goukei_focus { 3 } else { 1 };
        round_rect(target, btn_x, btn_y, btn_w, btn_h, 14, gbg, gborder, thick);
        draw_jp_text_centered(
            target,
            btn_x + btn_w / 2,
            btn_y + 16,
            "ゴウケイ",
            COLOR_BROWN,
            2,
        );

        // タイトルへ戻る
        let back_focus = self.focus == FOCUS_BACK;
        let back_color = if back_focus { COLOR_FOCUS } else { COLOR_GRAY };
        let back_label = if back_focus {
            "> タイトルにもどる <"
        } else {
            "タイトルにもどる"
        };
        draw_jp_text_centered(target, w / 2, h - 24, back_label, back_color, 1);
    }
}

fn draw_gingham<R: RenderTarget>(target: &mut R) {
    let w = target.width();
    let h = target.height();
    let stride = target.stride();
    let cell = 18usize;
    let buf = target.buffer_mut();
    for y in 0..h {
        let cy = y / cell;
        for x in 0..w {
            let cx = x / cell;
            let color = if (cx + cy).is_multiple_of(2) {
                COLOR_CHECK_A
            } else {
                COLOR_CHECK_B
            };
            let idx = y * stride + x;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    }
}

/// size: 0=小（カート用）, 1=大（ボタン用）
fn draw_veggie_icon<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, kind: usize, size: i32) {
    let s = if size == 0 { 1 } else { 2 };
    match kind {
        0 => draw_cabbage(target, cx, cy, s),
        1 => draw_tomato(target, cx, cy, s),
        2 => draw_carrot(target, cx, cy, s),
        3 => draw_fish(target, cx, cy, s),
        4 => draw_onion(target, cx, cy, s),
        5 => draw_banana(target, cx, cy, s),
        _ => fill_circle(target, cx, cy, 6 * s, COLOR_GREEN),
    }
}

fn draw_cabbage<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, s: i32) {
    let outline = 0x0020_6020;
    let dark = 0x0038_8838;
    let mid = 0x0058_B858;
    let light = 0x0080_D880;
    let pale = 0x00C0_F0C0;
    // 外葉
    fill_ellipse(target, cx, cy + s, 14 * s, 12 * s, mid);
    draw_ellipse(target, cx, cy + s, 14 * s, 12 * s, outline);
    // 中葉
    fill_ellipse(target, cx - 2 * s, cy, 10 * s, 9 * s, light);
    fill_ellipse(target, cx + 3 * s, cy + 2 * s, 8 * s, 7 * s, dark);
    // 芯
    fill_ellipse(target, cx, cy + s, 5 * s, 4 * s, pale);
    fill_circle(target, cx - 4 * s, cy - 2 * s, 2 * s, COLOR_WHITE);
}

fn draw_tomato<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, s: i32) {
    let body = 0x00E0_3030;
    let dark = 0x00A0_1818;
    let outline = 0x0060_1010;
    let leaf = 0x0030_A030;
    // 左の実
    fill_ellipse(target, cx - 7 * s, cy + 2 * s, 8 * s, 7 * s, body);
    draw_ellipse(target, cx - 7 * s, cy + 2 * s, 8 * s, 7 * s, outline);
    fill_circle(target, cx - 10 * s, cy - s, 2 * s, COLOR_WHITE);
    // 右の実
    fill_ellipse(target, cx + 7 * s, cy + 2 * s, 8 * s, 7 * s, body);
    draw_ellipse(target, cx + 7 * s, cy + 2 * s, 8 * s, 7 * s, outline);
    fill_circle(target, cx + 4 * s, cy - s, 2 * s, COLOR_WHITE);
    // ヘタ
    fill_ellipse(target, cx - 7 * s, cy - 5 * s, 4 * s, 2 * s, leaf);
    fill_ellipse(target, cx + 7 * s, cy - 5 * s, 4 * s, 2 * s, leaf);
    fill_rect(target, cx - 8 * s, cy - 8 * s, 2 * s, 4 * s, 0x0020_7020);
    fill_rect(target, cx + 6 * s, cy - 8 * s, 2 * s, 4 * s, 0x0020_7020);
    // 陰
    fill_ellipse(target, cx - 5 * s, cy + 5 * s, 3 * s, 2 * s, dark);
    fill_ellipse(target, cx + 9 * s, cy + 5 * s, 3 * s, 2 * s, dark);
}

fn draw_carrot<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, s: i32) {
    let body = COLOR_ORANGE;
    let dark = 0x00C0_6010;
    let leaf = 0x0030_B040;
    // 3本のにんじん
    for (ox, oy, ang_skew) in [(-10 * s, 2 * s, -1), (0, 0, 0), (10 * s, 2 * s, 1)] {
        let base_x = cx + ox;
        let base_y = cy + oy;
        for i in 0..14 * s {
            let t = i;
            let hw = (s + (14 * s - t) / 4).max(1);
            let xoff = ang_skew * t / (6 * s).max(1);
            fill_rect(
                target,
                base_x - hw + xoff,
                base_y - 6 * s + t,
                hw * 2,
                1,
                if t > 10 * s { dark } else { body },
            );
        }
        // 葉
        for (lx, ly) in [(-3 * s, -10 * s), (0, -12 * s), (3 * s, -10 * s)] {
            fill_ellipse(target, base_x + lx, base_y + ly, 2 * s, 4 * s, leaf);
        }
    }
}

fn draw_fish<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, s: i32) {
    let body = 0x0050_A0E0;
    let dark = 0x0030_7080;
    let outline = 0x0020_4058;
    let belly = 0x00C0_E8F8;
    // 胴体
    fill_ellipse(target, cx - 2 * s, cy + s, 12 * s, 7 * s, body);
    draw_ellipse(target, cx - 2 * s, cy + s, 12 * s, 7 * s, outline);
    // お腹
    fill_ellipse(target, cx - 2 * s, cy + 3 * s, 8 * s, 3 * s, belly);
    // 尾びれ
    for i in 0..8 * s {
        let hw = i / 2 + s;
        fill_rect(
            target,
            cx + 10 * s + i / 2,
            cy + s - hw,
            s.max(1),
            hw * 2,
            if i < 3 * s { body } else { dark },
        );
    }
    // 背びれ
    fill_ellipse(target, cx - s, cy - 5 * s, 4 * s, 3 * s, dark);
    // 目
    fill_circle(target, cx - 8 * s, cy, 2 * s, COLOR_WHITE);
    fill_circle(target, cx - 8 * s, cy, s.max(1), outline);
    // 口
    fill_rect(target, cx - 13 * s, cy + 2 * s, 3 * s, s.max(1), outline);
}

fn draw_onion<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, s: i32) {
    let skin = 0x00D0_B070;
    let dark = 0x00A0_8040;
    let outline = 0x0060_4820;
    let root = 0x00E8_D8A0;
    let sprout = 0x0040_B040;
    // 本体
    fill_ellipse(target, cx, cy + 2 * s, 11 * s, 12 * s, skin);
    draw_ellipse(target, cx, cy + 2 * s, 11 * s, 12 * s, outline);
    // 縦筋
    for dx in [-4 * s, 0, 4 * s] {
        for dy in -6 * s..8 * s {
            if dx * dx / 4 + dy * dy / 8 < 20 * s * s {
                put_pixel(target, cx + dx, cy + dy, dark);
            }
        }
    }
    // ハイライト
    fill_ellipse(target, cx - 4 * s, cy - 2 * s, 3 * s, 4 * s, 0x00F0_E0B0);
    // 芽
    fill_ellipse(target, cx - s, cy - 12 * s, s, 5 * s, sprout);
    fill_ellipse(target, cx + 2 * s, cy - 11 * s, s, 4 * s, sprout);
    // 根
    for (ox, oy) in [(-3 * s, 14 * s), (0, 15 * s), (3 * s, 14 * s)] {
        fill_rect(target, cx + ox, cy + oy, s, 3 * s, root);
    }
}

fn draw_banana<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, s: i32) {
    let body = 0x00F0_D040;
    let dark = 0x00D0_A020;
    let outline = 0x0080_6020;
    let tip = 0x0050_4030;
    // 湾曲したバナナ本体（楕円を重ねてカーブを表現）
    for (ox, oy, rx, ry) in [
        (-4 * s, 4 * s, 5 * s, 8 * s),
        (0, 0, 5 * s, 9 * s),
        (4 * s, -3 * s, 5 * s, 8 * s),
    ] {
        fill_ellipse(target, cx + ox, cy + oy, rx, ry, body);
        draw_ellipse(target, cx + ox, cy + oy, rx, ry, outline);
    }
    // 内側のハイライト
    fill_ellipse(target, cx - s, cy + s, 2 * s, 5 * s, 0x00FF_E880);
    // 両端の茶色いヘタ
    fill_ellipse(target, cx - 6 * s, cy + 10 * s, 2 * s, 2 * s, tip);
    fill_ellipse(target, cx + 7 * s, cy - 9 * s, 2 * s, 2 * s, tip);
    // 筋
    for dy in -4 * s..6 * s {
        put_pixel(target, cx + dy / 3, cy + dy, dark);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_vegetable_shows_price_and_cart() {
        let mut game = SelfCheckoutGame::new();
        let action = InputState {
            action: true,
            ..Default::default()
        };
        game.update(&action, &InputState::default());
        assert_eq!(game.cart_len, 1);
        assert_eq!(game.cart[0], Some(0));
        assert_eq!(game.display, DisplayMode::ItemPrice(150));
    }

    #[test]
    fn goukei_shows_total() {
        let mut game = SelfCheckoutGame::new();
        let action = InputState {
            action: true,
            ..Default::default()
        };
        game.update(&action, &InputState::default());
        game.update(&InputState::default(), &action);

        game.focus = 1;
        game.update(&action, &InputState::default());
        game.update(&InputState::default(), &action);

        game.focus = FOCUS_GOUKEI;
        game.update(&action, &InputState::default());
        assert_eq!(game.display, DisplayMode::Total(250));
        assert_eq!(game.cart_total(), 250);
    }

    #[test]
    fn back_returns_to_title() {
        let mut game = SelfCheckoutGame::new();
        game.focus = FOCUS_BACK;
        let action = InputState {
            action: true,
            ..Default::default()
        };
        game.update(&action, &InputState::default());
        assert!(game.wants_title());
    }

    #[test]
    fn cart_caps_at_max() {
        let mut game = SelfCheckoutGame::new();
        let action = InputState {
            action: true,
            ..Default::default()
        };
        for _ in 0..MAX_CART + 3 {
            game.focus = 0;
            game.update(&action, &InputState::default());
            game.update(&InputState::default(), &action);
        }
        assert_eq!(game.cart_len, MAX_CART);
    }

    #[test]
    fn japanese_labels_are_used() {
        assert_eq!(VEGETABLES[0].name, "キャベツ");
        assert_eq!(VEGETABLES[2].name, "にんじん");
    }
}
