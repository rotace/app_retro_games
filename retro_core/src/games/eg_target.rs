//! `RenderTarget` を `embedded-graphics` の `DrawTarget` に適合させるアダプタ

use embedded_graphics::{
    pixelcolor::{Rgb888, RgbColor},
    prelude::{DrawTarget, OriginDimensions, Point, Size},
    Pixel,
};

use crate::RenderTarget;

use super::draw::put_pixel;

/// ARGB8888 (`0x00RRGGBB`) → Rgb888
pub(crate) fn to_rgb(color: u32) -> Rgb888 {
    Rgb888::new(
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
}

/// `RenderTarget` への描画先ラッパ
pub(crate) struct EgTarget<'a, R: RenderTarget> {
    target: &'a mut R,
}

impl<'a, R: RenderTarget> EgTarget<'a, R> {
    pub(crate) fn new(target: &'a mut R) -> Self {
        Self { target }
    }
}

impl<R: RenderTarget> OriginDimensions for EgTarget<'_, R> {
    fn size(&self) -> Size {
        Size::new(self.target.width() as u32, self.target.height() as u32)
    }
}

impl<R: RenderTarget> DrawTarget for EgTarget<'_, R> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            let argb = ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | (color.b() as u32);
            put_pixel(self.target, x, y, argb);
        }
        Ok(())
    }
}
