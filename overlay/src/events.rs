use std::time::Duration;

use crate::renderer::texture::MediaAsset;
use shared::models::OverlayAnchor;

#[derive(Debug)]
pub enum RuntimeEvent {
    InjectMeme {
        anchor: OverlayAnchor,
        asset: MediaAsset,
        duration: Duration,
    },
}
