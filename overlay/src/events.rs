use crate::renderer::texture::GifAnimation;
use shared::models::OverlayAnchor;

#[derive(Debug)]
pub enum RuntimeEvent {
    InjectMeme {
        anchor: OverlayAnchor,
        animation: GifAnimation,
    },
}
