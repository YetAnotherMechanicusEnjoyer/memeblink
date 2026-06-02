use shared::error::Result;
use shared::models::OverlayAnchor;
use winit::window::Window;

pub trait NativeOverlay {
    fn initialize_overlay(&self, window: &Window) -> Result<()>;

    fn configure_input_passthrough(&self, window: &Window, enable_passthrough: bool) -> Result<()>;

    fn update_anchor(
        &self,
        window: &Window,
        anchor: OverlayAnchor,
        target_width: u32,
        target_height: u32,
        custom_x: Option<i32>,
        custom_y: Option<i32>,
    ) -> Result<()>;
}
