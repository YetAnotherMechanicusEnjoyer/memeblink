pub mod decoder;
pub mod texture;

use shared::error::Result;
use shared::models::OverlayAnchor;
use texture::TextureFrame;

pub trait OverlayCanvas {
    fn clear(&mut self) -> Result<()>;

    fn draw_texture(&mut self, texture: &TextureFrame, anchor: OverlayAnchor) -> Result<()>;

    fn present(&mut self) -> Result<()>;
}

pub struct MockCanvas;

impl OverlayCanvas for MockCanvas {
    #[inline]
    fn clear(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn draw_texture(&mut self, _texture: &TextureFrame, _anchor: OverlayAnchor) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn present(&mut self) -> Result<()> {
        Ok(())
    }
}
