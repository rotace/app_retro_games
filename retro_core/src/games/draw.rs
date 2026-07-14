//! ゲーム共通の描画ヘルパと色定数
//!
//! 図形描画は `embedded-graphics` に委譲する。

use embedded_graphics::{
    prelude::*,
    primitives::{
        Circle, Ellipse, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
    },
};

use crate::RenderTarget;

use super::eg_target::{to_rgb, EgTarget};

// 色定数 (ARGB8888)
// ---------------------------------------------------------------------------
pub(crate) const COLOR_BG: u32 = 0x0010_1020;
pub(crate) const COLOR_WHITE: u32 = 0x00FF_FFFF;
pub(crate) const COLOR_GRAY: u32 = 0x0088_8888;
pub(crate) const COLOR_YELLOW: u32 = 0x00FF_CC00;
pub(crate) const COLOR_CYAN: u32 = 0x0000_E5E5;
pub(crate) const COLOR_RED: u32 = 0x00E5_3030;
pub(crate) const COLOR_GREEN: u32 = 0x0030_C050;
pub(crate) const COLOR_BLUE: u32 = 0x0030_60E0;
pub(crate) const COLOR_ORANGE: u32 = 0x00E0_8020;
pub(crate) const COLOR_PURPLE: u32 = 0x00A0_40C0;
pub(crate) const COLOR_DARK: u32 = 0x0020_2030;

pub(crate) fn clear<R: RenderTarget>(target: &mut R, color: u32) {
    let buf = target.buffer_mut();
    for p in buf.iter_mut() {
        *p = color;
    }
}

pub(crate) fn put_pixel<R: RenderTarget>(target: &mut R, x: i32, y: i32, color: u32) {
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

pub(crate) fn fill_rect<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    let mut eg = EgTarget::new(target);
    let _ = Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32))
        .into_styled(PrimitiveStyle::with_fill(to_rgb(color)))
        .draw(&mut eg);
}

pub(crate) fn draw_rect<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    let mut eg = EgTarget::new(target);
    let _ = Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32))
        .into_styled(PrimitiveStyle::with_stroke(to_rgb(color), 1))
        .draw(&mut eg);
}

pub(crate) fn fill_circle<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, r: i32, color: u32) {
    if r <= 0 {
        return;
    }
    let d = (r * 2) as u32;
    let mut eg = EgTarget::new(target);
    let _ = Circle::new(Point::new(cx - r, cy - r), d)
        .into_styled(PrimitiveStyle::with_fill(to_rgb(color)))
        .draw(&mut eg);
}

#[allow(dead_code)]
pub(crate) fn draw_circle<R: RenderTarget>(target: &mut R, cx: i32, cy: i32, r: i32, color: u32) {
    if r <= 0 {
        return;
    }
    let d = (r * 2) as u32;
    let mut eg = EgTarget::new(target);
    let _ = Circle::new(Point::new(cx - r, cy - r), d)
        .into_styled(PrimitiveStyle::with_stroke(to_rgb(color), 1))
        .draw(&mut eg);
}

pub(crate) fn fill_ellipse<R: RenderTarget>(
    target: &mut R,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    color: u32,
) {
    if rx <= 0 || ry <= 0 {
        return;
    }
    let mut eg = EgTarget::new(target);
    let _ = Ellipse::new(
        Point::new(cx - rx, cy - ry),
        Size::new((rx * 2) as u32, (ry * 2) as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(to_rgb(color)))
    .draw(&mut eg);
}

pub(crate) fn draw_ellipse<R: RenderTarget>(
    target: &mut R,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    color: u32,
) {
    if rx <= 0 || ry <= 0 {
        return;
    }
    let mut eg = EgTarget::new(target);
    let _ = Ellipse::new(
        Point::new(cx - rx, cy - ry),
        Size::new((rx * 2) as u32, (ry * 2) as u32),
    )
    .into_styled(PrimitiveStyle::with_stroke(to_rgb(color), 1))
    .draw(&mut eg);
}

#[allow(dead_code)]
pub(crate) fn fill_round_rect<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    radius: i32,
    color: u32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    let r = radius.max(0) as u32;
    let mut eg = EgTarget::new(target);
    let _ = RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32)),
        Size::new(r, r),
    )
    .into_styled(PrimitiveStyle::with_fill(to_rgb(color)))
    .draw(&mut eg);
}

#[allow(dead_code)] // 枠のみ描画が必要なゲーム向けに残す
pub(crate) fn draw_round_rect<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    radius: i32,
    color: u32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    let r = radius.max(0) as u32;
    let mut eg = EgTarget::new(target);
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(to_rgb(color))
        .stroke_width(1)
        .build();
    let _ = RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32)),
        Size::new(r, r),
    )
    .into_styled(style)
    .draw(&mut eg);
}

/// 塗り＋枠の角丸矩形を一度に描画
#[allow(clippy::too_many_arguments)]
pub(crate) fn round_rect<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    radius: i32,
    fill: u32,
    stroke: u32,
    stroke_width: u32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    let r = radius.max(0) as u32;
    let mut eg = EgTarget::new(target);
    let style = PrimitiveStyleBuilder::new()
        .fill_color(to_rgb(fill))
        .stroke_color(to_rgb(stroke))
        .stroke_width(stroke_width.max(1))
        .build();
    let _ = RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32)),
        Size::new(r, r),
    )
    .into_styled(style)
    .draw(&mut eg);
}

/// 7セグメント風の数字1桁
pub(crate) fn draw_digit_7seg<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    digit: u8,
    color: u32,
    scale: i32,
) {
    let s = scale.max(1);
    const SEGS: [u8; 10] = [
        0b011_1111, // 0
        0b000_0110, // 1
        0b101_1011, // 2
        0b100_1111, // 3
        0b110_0110, // 4
        0b110_1101, // 5
        0b111_1101, // 6
        0b000_0111, // 7
        0b111_1111, // 8
        0b110_1111, // 9
    ];
    let mask = SEGS.get(digit as usize).copied().unwrap_or(0);
    let t = s;
    let gw = 6 * s;
    let gh = 10 * s;
    let mut seg = |on: bool, x0: i32, y0: i32, w: i32, h: i32| {
        if on {
            fill_rect(target, x0, y0, w, h, color);
        }
    };
    seg(mask & 1 != 0, x + t, y, gw - t, t);
    seg(mask & 2 != 0, x + gw, y + t, t, gh / 2 - t);
    seg(mask & 4 != 0, x + gw, y + gh / 2 + t, t, gh / 2 - t);
    seg(mask & 8 != 0, x + t, y + gh, gw - t, t);
    seg(mask & 16 != 0, x, y + gh / 2 + t, t, gh / 2 - t);
    seg(mask & 32 != 0, x, y + t, t, gh / 2 - t);
    seg(mask & 64 != 0, x + t, y + gh / 2, gw - t, t);
}

/// 7セグメント風の数値描画。戻り値は描画幅。
pub(crate) fn draw_number_7seg<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    mut n: u32,
    color: u32,
    scale: i32,
) -> i32 {
    let s = scale.max(1);
    let digit_w = 8 * s;
    let mut digits = [0u8; 10];
    let mut len = 0;
    if n == 0 {
        digits[0] = 0;
        len = 1;
    } else {
        while n > 0 && len < digits.len() {
            digits[len] = (n % 10) as u8;
            n /= 10;
            len += 1;
        }
    }
    for i in 0..len {
        let d = digits[len - 1 - i];
        draw_digit_7seg(target, x + i as i32 * digit_w, y, d, color, s);
    }
    len as i32 * digit_w
}

/// 5x7 簡易ビットマップフォント（ASCII の一部）— Breakout/Tetris 向け
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

pub(crate) fn draw_char<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    c: u8,
    color: u32,
    scale: i32,
) {
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

pub(crate) fn draw_text<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    text: &str,
    color: u32,
    scale: i32,
) {
    let s = scale.max(1);
    let advance = 6 * s;
    for (i, c) in text.bytes().enumerate() {
        draw_char(target, x + i as i32 * advance, y, c, color, s);
    }
}

pub(crate) fn text_width(text: &str, scale: i32) -> i32 {
    let s = scale.max(1);
    text.len() as i32 * 6 * s
}

pub(crate) fn draw_text_centered<R: RenderTarget>(
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

pub(crate) fn format_u32(buf: &mut [u8], mut n: u32) -> &str {
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

pub(crate) fn draw_score_line<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    label: &str,
    value: u32,
) {
    draw_text(target, x, y, label, COLOR_GRAY, 1);
    let mut num = [0u8; 10];
    let s = format_u32(&mut num, value);
    draw_text(target, x + text_width(label, 1) + 6, y, s, COLOR_WHITE, 1);
}
