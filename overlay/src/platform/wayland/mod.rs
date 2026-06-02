use crate::constants::WAYLAND_NAMESPACE;
use crate::platform::traits::NativeOverlay;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use shared::error::{MemeBlinkError, Result};
use shared::models::OverlayAnchor;
use std::process::{Command, Stdio};
use winit::window::Window;

pub struct WaylandOverlayEngine {
    runtime_namespace: &'static str,
}

impl WaylandOverlayEngine {
    #[inline]
    pub const fn new() -> Self {
        Self {
            runtime_namespace: WAYLAND_NAMESPACE,
        }
    }

    fn extract_display_ptr(&self, window: &Window) -> Result<*mut std::ffi::c_void> {
        let handle = window
            .display_handle()
            .map_err(|e| MemeBlinkError::WaylandInitialization(e.to_string()))?;

        match handle.as_raw() {
            RawDisplayHandle::Wayland(wayland_handle) => Ok(wayland_handle.display.as_ptr()),
            _ => Err(MemeBlinkError::WaylandInitialization(
                "The current graphics context is not a native Wayland display".to_string(),
            )),
        }
    }

    fn extract_surface_ptr(&self, window: &Window) -> Result<*mut std::ffi::c_void> {
        let handle = window
            .window_handle()
            .map_err(|e| MemeBlinkError::WaylandInitialization(e.to_string()))?;

        match handle.as_raw() {
            RawWindowHandle::Wayland(wayland_handle) => Ok(wayland_handle.surface.as_ptr()),
            _ => Err(MemeBlinkError::WaylandInitialization(
                "The current window context is not a native Wayland surface".to_string(),
            )),
        }
    }
}

impl NativeOverlay for WaylandOverlayEngine {
    fn initialize_overlay(&self, _window: &Window) -> Result<()> {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let rules = [
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, float on",
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, pin on",
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, no_focus on",
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, no_shadow on",
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, no_anim on",
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, no_blur on",
                "windowrule = match:class ^(memeblink)$, match:title ^(MemeBlink Overlay)$, border_size 0",
            ];
            for rule in rules {
                Command::new("hyprctl")
                    .args(["keyword", rule])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .ok();
            }
        }
        Ok(())
    }

    fn configure_input_passthrough(&self, window: &Window, enable_passthrough: bool) -> Result<()> {
        let hittest_active = !enable_passthrough;
        window.set_cursor_hittest(hittest_active).map_err(|e| {
            MemeBlinkError::WaylandInitialization(format!("Failed to set cursor hittest: {}", e))
        })?;
        Ok(())
    }

    fn update_anchor(
        &self,
        window: &Window,
        anchor: OverlayAnchor,
        target_width: u32,
        target_height: u32,
        custom_x: Option<i32>,
        custom_y: Option<i32>,
    ) -> Result<()> {
        let monitor = window
            .current_monitor()
            .or_else(|| window.primary_monitor())
            .or_else(|| window.available_monitors().next());

        let Some(monitor) = monitor else {
            return Ok(());
        };
        let monitor_size = monitor.size();
        let monitor_pos = monitor.position();

        let (mut target_x, mut target_y) = match anchor {
            OverlayAnchor::TopLeft => (monitor_pos.x, monitor_pos.y),
            OverlayAnchor::TopRight => (
                monitor_pos.x + monitor_size.width.saturating_sub(target_width) as i32,
                monitor_pos.y,
            ),
            OverlayAnchor::BottomLeft => (
                monitor_pos.x,
                monitor_pos.y + monitor_size.height.saturating_sub(target_height) as i32,
            ),
            OverlayAnchor::BottomRight => (
                monitor_pos.x + monitor_size.width.saturating_sub(target_width) as i32,
                monitor_pos.y + monitor_size.height.saturating_sub(target_height) as i32,
            ),
            OverlayAnchor::Center => (
                monitor_pos.x + (monitor_size.width.saturating_sub(target_width) / 2) as i32,
                monitor_pos.y + (monitor_size.height.saturating_sub(target_height) / 2) as i32,
            ),
        };

        if let Some(cx) = custom_x {
            target_x = monitor_pos.x + cx;
        }
        if let Some(cy) = custom_y {
            target_y = monitor_pos.y + cy;
        }

        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let resize_args = format!(
                "exact {} {},title:MemeBlink Overlay",
                target_width, target_height
            );
            let move_args = format!("exact {} {},title:MemeBlink Overlay", target_x, target_y);

            for _ in 0..3 {
                let output = Command::new("hyprctl")
                    .args(["dispatch", "resizewindowpixel", &resize_args])
                    .output();
                if let Ok(out) = output {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);

                    if !stderr.contains("no window") && !stdout.contains("no window") {
                        Command::new("hyprctl")
                            .args(["dispatch", "movewindowpixel", &move_args])
                            .stdout(Stdio::null())
                            .status()
                            .ok();
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        } else {
            let _ = window
                .request_inner_size(winit::dpi::PhysicalSize::new(target_width, target_height));
            window.set_outer_position(winit::dpi::PhysicalPosition::new(target_x, target_y));
        }
        Ok(())
    }
}
