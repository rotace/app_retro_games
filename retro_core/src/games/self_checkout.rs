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
    draw_rect, draw_text, draw_text_centered, fill_rect, format_u32, text_width, COLOR_DARK,
    COLOR_GRAY, COLOR_GREEN, COLOR_ORANGE, COLOR_RED, COLOR_WHITE,
};
use super::just_pressed;

/// 背景のギンガム（水色チェック）
const COLOR_CHECK_A: u32 = 0x00B8_D4E8;
const COLOR_CHECK_B: u32 = 0x00E8_F4FA;
/// クリーム系ボタン背景
const COLOR_CREAM: u32 = 0x00F5_E6C8;
/// ゴウケイボタンの金色
const COLOR_GOLD: u32 = 0x00E8_C040;
/// 枠線・文字（茶系）
const COLOR_BROWN: u32 = 0x0040_3020;
/// フォーカス強調
const COLOR_FOCUS: u32 = 0x00E0_5050;

const MAX_CART: usize = 12;
const VEG_COLS: usize = 3;
const VEG_ROWS: usize = 2;
const VEG_COUNT: usize = 6;

/// フォーカス: 0..5 = 野菜, 6 = ゴウケイ, 7 = タイトルへ戻る
const FOCUS_GOUKEI: usize = 6;
const FOCUS_BACK: usize = 7;

#[derive(Debug, Clone, Copy)]
struct Vegetable {
    name: &'static str,
    price: u32,
    /// アイコン描画用の色
    color: u32,
}

const VEGETABLES: [Vegetable; VEG_COUNT] = [
    Vegetable {
        name: "CABBAGE",
        price: 150,
        color: 0x0050_A050,
    },
    Vegetable {
        name: "TOMATO",
        price: 100,
        color: COLOR_RED,
    },
    Vegetable {
        name: "CARROT",
        price: 80,
        color: COLOR_ORANGE,
    },
    Vegetable {
        name: "BROCCOLI",
        price: 130,
        color: COLOR_GREEN,
    },
    Vegetable {
        name: "ONION",
        price: 60,
        color: 0x00C0_A060,
    },
    Vegetable {
        name: "PEPPER",
        price: 90,
        color: 0x0040_B040,
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

        // 野菜グリッド (0..5) / ゴウケイ(6) / 戻る(7) の簡易ナビ
        match self.focus {
            0..=5 => {
                let col = (self.focus % VEG_COLS) as i32;
                let row = (self.focus / VEG_COLS) as i32;
                let new_col = col + dx;
                let new_row = row + dy;

                if new_col < 0 {
                    // 左端から右カラムへ（ゴウケイ or 戻る）
                    if row == 0 {
                        self.focus = FOCUS_GOUKEI;
                    } else {
                        self.focus = FOCUS_BACK;
                    }
                } else if new_col >= VEG_COLS as i32 {
                    if row == 0 {
                        self.focus = FOCUS_GOUKEI;
                    } else {
                        self.focus = FOCUS_BACK;
                    }
                } else if new_row < 0 {
                    // 上端はそのまま
                } else if new_row >= VEG_ROWS as i32 {
                    self.focus = FOCUS_BACK;
                } else {
                    self.focus = (new_row as usize) * VEG_COLS + (new_col as usize);
                }
            }
            FOCUS_GOUKEI => {
                if dx < 0 || dy > 0 {
                    // 左 or 下 → 野菜側 / 戻る
                    if dy > 0 {
                        self.focus = FOCUS_BACK;
                    } else {
                        self.focus = 2; // 右上の野菜付近
                    }
                } else if dy < 0 {
                    // 上はゴウケイのまま
                } else if dx > 0 {
                    // 右は無視
                }
            }
            FOCUS_BACK => {
                if dy < 0 {
                    self.focus = FOCUS_GOUKEI;
                } else if dx < 0 {
                    self.focus = 5; // 右下の野菜
                } else if dx > 0 {
                    // 右は無視
                } else if dy > 0 {
                    // 下は無視
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

        // --- 左上: 値段表示 ---
        let disp_x = 16;
        let disp_y = 16;
        let disp_w = w * 55 / 100 - 24;
        let disp_h = 56;
        fill_rect(target, disp_x, disp_y, disp_w, disp_h, COLOR_WHITE);
        draw_rect(target, disp_x, disp_y, disp_w, disp_h, COLOR_BROWN);
        draw_rect(
            target,
            disp_x + 1,
            disp_y + 1,
            disp_w - 2,
            disp_h - 2,
            COLOR_BROWN,
        );

        draw_text(target, disp_x + 10, disp_y + 20, "YASAI:", COLOR_BROWN, 2);
        match self.display {
            DisplayMode::Empty => {
                draw_text(
                    target,
                    disp_x + disp_w - 80,
                    disp_y + 18,
                    "--- YEN",
                    COLOR_GRAY,
                    2,
                );
            }
            DisplayMode::ItemPrice(price) | DisplayMode::Total(price) => {
                let mut num = [0u8; 10];
                let s = format_u32(&mut num, price);
                let label_w = text_width(s, 3) + text_width(" YEN", 2);
                let px = disp_x + disp_w - label_w - 12;
                draw_text(target, px, disp_y + 12, s, COLOR_BROWN, 3);
                draw_text(
                    target,
                    px + text_width(s, 3) + 4,
                    disp_y + 22,
                    "YEN",
                    COLOR_BROWN,
                    2,
                );
            }
        }

        // --- 左下: お野菜ボタン ---
        let grid_x = 16;
        let grid_y = disp_y + disp_h + 16;
        let grid_w = disp_w;
        let grid_h = h - grid_y - 40;
        let cell_w = (grid_w - 16) / VEG_COLS as i32;
        let cell_h = (grid_h - 12) / VEG_ROWS as i32;

        for (i, veg) in VEGETABLES.iter().enumerate() {
            let col = (i % VEG_COLS) as i32;
            let row = (i / VEG_COLS) as i32;
            let bx = grid_x + 4 + col * cell_w;
            let by = grid_y + 4 + row * cell_h;
            let bw = cell_w - 8;
            let bh = cell_h - 8;
            let focused = self.focus == i;

            fill_rect(target, bx, by, bw, bh, COLOR_CREAM);
            let border = if focused { COLOR_FOCUS } else { COLOR_BROWN };
            draw_rect(target, bx, by, bw, bh, border);
            if focused {
                draw_rect(target, bx + 1, by + 1, bw - 2, bh - 2, COLOR_FOCUS);
            }

            draw_veggie_icon(target, bx + bw / 2, by + bh / 2 - 14, i, veg.color);

            let name_scale = 1;
            draw_text_centered(
                target,
                (bx + bw / 2) as usize,
                (by + bh - 28) as usize,
                veg.name,
                COLOR_BROWN,
                name_scale,
            );
            let mut pbuf = [0u8; 10];
            let ps = format_u32(&mut pbuf, veg.price);
            // "150YEN" を描画
            let mut price_text = [0u8; 16];
            let plen = ps.len().min(10);
            price_text[..plen].copy_from_slice(ps.as_bytes());
            let yen = b"YEN";
            price_text[plen..plen + 3].copy_from_slice(yen);
            let price_str = std::str::from_utf8(&price_text[..plen + 3]).unwrap_or("");
            draw_text_centered(
                target,
                (bx + bw / 2) as usize,
                (by + bh - 16) as usize,
                price_str,
                COLOR_DARK,
                1,
            );
        }

        // --- 右上: カート一覧 ---
        let cart_x = w * 55 / 100 + 8;
        let cart_y = 16;
        let cart_w = w - cart_x - 16;
        let cart_h = h - 120;
        fill_rect(target, cart_x, cart_y, cart_w, cart_h, COLOR_CREAM);
        draw_rect(target, cart_x, cart_y, cart_w, cart_h, COLOR_BROWN);

        draw_text_centered(
            target,
            (cart_x + cart_w / 2) as usize,
            (cart_y + 12) as usize,
            "CART",
            COLOR_BROWN,
            2,
        );

        let list_top = cart_y + 40;
        let line_h = 28;
        let visible = ((cart_h - 48) / line_h).max(1) as usize;
        let start = self.cart_len.saturating_sub(visible);
        for (row, ci) in (start..self.cart_len).enumerate() {
            if let Some(idx) = self.cart[ci] {
                let veg = &VEGETABLES[idx];
                let ly = list_top + row as i32 * line_h;
                // 小さなアイコン
                draw_veggie_icon(target, cart_x + 18, ly + 10, idx, veg.color);
                draw_text(target, cart_x + 36, ly + 6, veg.name, COLOR_BROWN, 1);
                let mut pbuf = [0u8; 10];
                let ps = format_u32(&mut pbuf, veg.price);
                let mut price_text = [0u8; 16];
                let plen = ps.len().min(10);
                price_text[..plen].copy_from_slice(ps.as_bytes());
                price_text[plen..plen + 3].copy_from_slice(b"YEN");
                let price_str = std::str::from_utf8(&price_text[..plen + 3]).unwrap_or("");
                let tw = text_width(price_str, 1);
                draw_text(
                    target,
                    cart_x + cart_w - tw - 10,
                    ly + 6,
                    price_str,
                    COLOR_BROWN,
                    1,
                );
            }
        }

        // --- 右下: ゴウケイボタン ---
        let btn_x = cart_x;
        let btn_y = cart_y + cart_h + 12;
        let btn_w = cart_w;
        let btn_h = 48;
        let goukei_focus = self.focus == FOCUS_GOUKEI;
        fill_rect(target, btn_x, btn_y, btn_w, btn_h, COLOR_GOLD);
        let gborder = if goukei_focus {
            COLOR_FOCUS
        } else {
            COLOR_BROWN
        };
        draw_rect(target, btn_x, btn_y, btn_w, btn_h, gborder);
        if goukei_focus {
            draw_rect(
                target,
                btn_x + 1,
                btn_y + 1,
                btn_w - 2,
                btn_h - 2,
                COLOR_FOCUS,
            );
        }
        draw_text_centered(
            target,
            (btn_x + btn_w / 2) as usize,
            (btn_y + 14) as usize,
            "GOUKEI",
            COLOR_BROWN,
            2,
        );

        // タイトルへ戻る
        let back_focus = self.focus == FOCUS_BACK;
        let back_color = if back_focus { COLOR_FOCUS } else { COLOR_GRAY };
        draw_text_centered(
            target,
            (w / 2) as usize,
            (h - 22) as usize,
            "BACK: TITLE",
            back_color,
            1,
        );

        // 操作ヒント（初回など空のとき）
        if self.cart_len == 0 && matches!(self.display, DisplayMode::Empty) {
            draw_text(target, 16, h - 22, "ARROWS+ACTION", COLOR_GRAY, 1);
        }
    }
}

fn draw_gingham<R: RenderTarget>(target: &mut R) {
    let w = target.width();
    let h = target.height();
    let stride = target.stride();
    let cell = 20usize;
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

fn draw_veggie_icon<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, kind: usize, color: u32) {
    match kind {
        0 => {
            // キャベツ: 同心円
            fill_circle(target, cx, cy, 12, color);
            fill_circle(target, cx, cy, 7, 0x0070_C070);
            fill_circle(target, cx, cy, 3, COLOR_WHITE);
        }
        1 => {
            // トマト: 2つの丸
            fill_circle(target, cx - 6, cy + 2, 7, color);
            fill_circle(target, cx + 6, cy + 2, 7, color);
            fill_rect(target, cx - 2, cy - 8, 4, 5, COLOR_GREEN);
        }
        2 => {
            // にんじん: 三角形風
            for i in 0..12 {
                let hw = i / 2 + 1;
                fill_rect(target, cx - hw, cy - 8 + i, hw * 2, 1, color);
            }
            fill_rect(target, cx - 2, cy - 12, 4, 5, COLOR_GREEN);
        }
        3 => {
            // ブロッコリー: 房 + 茎
            fill_circle(target, cx, cy - 4, 9, color);
            fill_circle(target, cx - 7, cy, 6, color);
            fill_circle(target, cx + 7, cy, 6, color);
            fill_rect(target, cx - 3, cy + 4, 6, 10, 0x0060_A040);
        }
        4 => {
            // タマネギ: 楕円 + 芽
            fill_circle(target, cx, cy + 2, 10, color);
            fill_rect(target, cx - 1, cy - 12, 2, 6, COLOR_GREEN);
            fill_rect(target, cx + 2, cy - 10, 2, 4, COLOR_GREEN);
        }
        5 => {
            // ピーマン: 縦長の2個
            fill_circle(target, cx - 6, cy, 6, color);
            fill_rect(target, cx - 10, cy - 2, 8, 10, color);
            fill_circle(target, cx + 6, cy, 6, color);
            fill_rect(target, cx + 2, cy - 2, 8, 10, color);
            fill_rect(target, cx - 7, cy - 8, 3, 4, 0x0030_8030);
            fill_rect(target, cx + 5, cy - 8, 3, 4, 0x0030_8030);
        }
        _ => fill_circle(target, cx, cy, 8, color),
    }
}

fn fill_circle<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, r: i32, color: u32) {
    let r2 = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r2 {
                // put_pixel 相当
                let x = cx + dx;
                let y = cy + dy;
                if x < 0 || y < 0 {
                    continue;
                }
                let x = x as usize;
                let y = y as usize;
                let w = target.width();
                let h = target.height();
                let stride = target.stride();
                if x >= w || y >= h {
                    continue;
                }
                let buf = target.buffer_mut();
                let idx = y * stride + x;
                if idx < buf.len() {
                    buf[idx] = color;
                }
            }
        }
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
        // focus 0 = CABBAGE
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
        // CABBAGE 150
        game.update(&action, &InputState::default());
        game.update(&InputState::default(), &action);

        // TOMATO へ移動して追加
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
}
