use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SizeConfig {
    Fixed(u32),
    Auto(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayEvent {
    pub image_path: String,
    pub text: Option<String>,
    pub duration_ms: u32,
    pub anchor: OverlayAnchor,
    #[serde(default)]
    pub width: Option<SizeConfig>,
    #[serde(default)]
    pub height: Option<SizeConfig>,
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
}
