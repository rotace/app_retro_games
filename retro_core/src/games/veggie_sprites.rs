//! セルフレジ用お野菜・果物スプライト（いらすとや素材を縮小埋め込み）
//!
//! 素材出典: いらすとや（みふねたかし）。著作権は放棄されていません。
//! 詳細: `assets/irasutoya/ATTRIBUTION.txt`

use super::draw::put_pixel;
use crate::RenderTarget;

pub(crate) struct Sprite {
    pub width: i32,
    pub height: i32,
    /// 行優先 RGBA8888
    pub pixels: &'static [u8],
}

static CABBAGE_24_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/cabbage_24.rgba"
));
pub(crate) static CABBAGE_24: Sprite = Sprite {
    width: 24,
    height: 24,
    pixels: CABBAGE_24_RGBA,
};

static CABBAGE_48_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/cabbage_48.rgba"
));
pub(crate) static CABBAGE_48: Sprite = Sprite {
    width: 48,
    height: 48,
    pixels: CABBAGE_48_RGBA,
};

static TOMATO_24_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/tomato_24.rgba"
));
pub(crate) static TOMATO_24: Sprite = Sprite {
    width: 24,
    height: 24,
    pixels: TOMATO_24_RGBA,
};

static TOMATO_48_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/tomato_48.rgba"
));
pub(crate) static TOMATO_48: Sprite = Sprite {
    width: 48,
    height: 48,
    pixels: TOMATO_48_RGBA,
};

static CARROT_24_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/carrot_24.rgba"
));
pub(crate) static CARROT_24: Sprite = Sprite {
    width: 24,
    height: 24,
    pixels: CARROT_24_RGBA,
};

static CARROT_48_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/carrot_48.rgba"
));
pub(crate) static CARROT_48: Sprite = Sprite {
    width: 48,
    height: 48,
    pixels: CARROT_48_RGBA,
};

static FISH_24_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/fish_24.rgba"
));
pub(crate) static FISH_24: Sprite = Sprite {
    width: 24,
    height: 24,
    pixels: FISH_24_RGBA,
};

static FISH_48_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/fish_48.rgba"
));
pub(crate) static FISH_48: Sprite = Sprite {
    width: 48,
    height: 48,
    pixels: FISH_48_RGBA,
};

static ONION_24_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/onion_24.rgba"
));
pub(crate) static ONION_24: Sprite = Sprite {
    width: 24,
    height: 24,
    pixels: ONION_24_RGBA,
};

static ONION_48_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/onion_48.rgba"
));
pub(crate) static ONION_48: Sprite = Sprite {
    width: 48,
    height: 48,
    pixels: ONION_48_RGBA,
};

static BANANA_24_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/banana_24.rgba"
));
pub(crate) static BANANA_24: Sprite = Sprite {
    width: 24,
    height: 24,
    pixels: BANANA_24_RGBA,
};

static BANANA_48_RGBA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/irasutoya/banana_48.rgba"
));
pub(crate) static BANANA_48: Sprite = Sprite {
    width: 48,
    height: 48,
    pixels: BANANA_48_RGBA,
};

/// kind: 0..5, size: 0=小(24), 1=大(48)
pub(crate) fn veggie_sprite(kind: usize, size: i32) -> Option<&'static Sprite> {
    let large = size != 0;
    Some(match (kind, large) {
        (0, false) => &CABBAGE_24,
        (0, true) => &CABBAGE_48,
        (1, false) => &TOMATO_24,
        (1, true) => &TOMATO_48,
        (2, false) => &CARROT_24,
        (2, true) => &CARROT_48,
        (3, false) => &FISH_24,
        (3, true) => &FISH_48,
        (4, false) => &ONION_24,
        (4, true) => &ONION_48,
        (5, false) => &BANANA_24,
        (5, true) => &BANANA_48,
        _ => return None,
    })
}

/// 中心 (cx, cy) にスプライトを描画。alpha < 128 はスキップ（簡易）。
pub(crate) fn blit_sprite_centered<R: RenderTarget>(
    target: &mut R,
    cx: i32,
    cy: i32,
    sprite: &Sprite,
) {
    let x0 = cx - sprite.width / 2;
    let y0 = cy - sprite.height / 2;
    blit_sprite(target, x0, y0, sprite);
}

pub(crate) fn blit_sprite<R: RenderTarget>(target: &mut R, x0: i32, y0: i32, sprite: &Sprite) {
    let w = sprite.width;
    let h = sprite.height;
    let px = sprite.pixels;
    let expected = (w * h * 4) as usize;
    if px.len() < expected {
        return;
    }
    for row in 0..h {
        for col in 0..w {
            let i = ((row * w + col) * 4) as usize;
            let r = px[i] as u32;
            let g = px[i + 1] as u32;
            let b = px[i + 2] as u32;
            let a = px[i + 3];
            if a < 128 {
                continue;
            }
            let color = (r << 16) | (g << 8) | b;
            put_pixel(target, x0 + col, y0 + row, color);
        }
    }
}
