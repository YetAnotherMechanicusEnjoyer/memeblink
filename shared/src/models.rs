use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayEvent {
    pub image_path: String,
    pub text: Option<String>,
    pub duration_ms: u32,
    pub anchor: OverlayAnchor,
}
