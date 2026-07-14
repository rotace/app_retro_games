//! 日本語テキスト描画（`u8g2-fonts` + `embedded-graphics`）
//!
//! U8g2 の `b12_t_japanese3` / `b16_t_japanese3` を利用する。
//! 欠落グリフがあってもパニックせず、描画をスキップする。

use embedded_graphics::prelude::Point;
use u8g2_fonts::{
    fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

use crate::RenderTarget;

use super::eg_target::{to_rgb, EgTarget};

/// 通常サイズ（12px）
fn font_normal() -> FontRenderer {
    FontRenderer::new::<fonts::u8g2_font_b12_t_japanese3>()
}

/// 大きめ（16px）— ゴウケイボタンなど
fn font_large() -> FontRenderer {
    FontRenderer::new::<fonts::u8g2_font_b16_t_japanese3>()
}

fn pick_font(scale: i32) -> FontRenderer {
    if scale >= 2 {
        font_large()
    } else {
        font_normal()
    }
}

/// 日本語文字列の描画幅（advance）
pub(crate) fn jp_text_width(text: &str, scale: i32) -> i32 {
    let font = pick_font(scale);
    match font.get_rendered_dimensions(text, Point::zero(), VerticalPosition::Top) {
        Ok(dims) => dims.advance.x,
        Err(_) => 0,
    }
}

/// 日本語文字列を描画。`scale >= 2` で大きめフォントを使う。
pub(crate) fn draw_jp_text<R: RenderTarget>(
    target: &mut R,
    x: i32,
    y: i32,
    text: &str,
    color: u32,
    scale: i32,
) {
    let font = pick_font(scale);
    let mut eg = EgTarget::new(target);
    // 憲法: panic 禁止。欠落グリフ等は無視する。
    let _ = font.render(
        text,
        Point::new(x, y),
        VerticalPosition::Top,
        FontColor::Transparent(to_rgb(color)),
        &mut eg,
    );
}

pub(crate) fn draw_jp_text_centered<R: RenderTarget>(
    target: &mut R,
    cx: i32,
    y: i32,
    text: &str,
    color: u32,
    scale: i32,
) {
    let font = pick_font(scale);
    let mut eg = EgTarget::new(target);
    let _ = font.render_aligned(
        text,
        Point::new(cx, y),
        VerticalPosition::Top,
        HorizontalAlignment::Center,
        FontColor::Transparent(to_rgb(color)),
        &mut eg,
    );
}
