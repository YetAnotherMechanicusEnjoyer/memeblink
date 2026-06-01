use shared::models::OverlayAnchor;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;

use crate::events::RuntimeEvent;
use crate::platform::traits::NativeOverlay;
use crate::renderer::texture::GifAnimation;

pub struct MemeBlinkOverlayApp<O, T>
where
    O: NativeOverlay,
{
    platform_engine: O,
    window: Option<Arc<Window>>,
    context: Option<softbuffer::Context<Arc<Window>>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    active_animation: Option<GifAnimation>,
    active_anchor: Option<OverlayAnchor>,
    _event_marker: PhantomData<T>,
}

impl<O, T> MemeBlinkOverlayApp<O, T>
where
    O: NativeOverlay,
{
    #[inline]
    pub fn new(platform_engine: O) -> Self {
        Self {
            platform_engine,
            window: None,
            context: None,
            surface: None,
            active_animation: None,
            active_anchor: None,
            _event_marker: PhantomData,
        }
    }

    fn render_frame(&mut self) {
        let (Some(surface), Some(window)) = (self.surface.as_mut(), self.window.as_ref()) else {
            return;
        };
        let Some(animation) = self.active_animation.as_ref() else {
            return;
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let w = NonZeroU32::new(size.width).unwrap();
        let h = NonZeroU32::new(size.height).unwrap();
        if surface.resize(w, h).is_err() {
            return;
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(b) => b,
            Err(_) => return,
        };

        buffer.fill(0);

        let frame = animation.current_frame();
        let target_x = 0;
        let target_y = 0;

        for row in 0..frame.height {
            let surface_row = row + target_y;
            if surface_row >= size.height {
                break;
            }

            let surface_idx = (surface_row * size.width + target_x) as usize;
            let texture_idx = (row * frame.width) as usize;
            let copy_len = (frame.width as usize).min(buffer.len() - surface_idx);

            if copy_len > 0 {
                buffer[surface_idx..surface_idx + copy_len]
                    .copy_from_slice(&frame.pixels[texture_idx..texture_idx + copy_len]);
            }
        }
        let _ = buffer.present();
    }
}

impl<O> ApplicationHandler<RuntimeEvent> for MemeBlinkOverlayApp<O, RuntimeEvent>
where
    O: NativeOverlay,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut window_attributes = WindowAttributes::default()
            .with_title("MemeBlink Overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop);

        #[cfg(target_os = "linux")]
        {
            window_attributes = window_attributes.with_name("memeblink", "");
        }

        if let Ok(new_window) = event_loop.create_window(window_attributes) {
            let _ = self.platform_engine.initialize_overlay(&new_window);
            let _ = self
                .platform_engine
                .configure_input_passthrough(&new_window, true);

            let window_arc = Arc::new(new_window);

            if let Ok(context) = softbuffer::Context::new(window_arc.clone())
                && let Ok(surface) = softbuffer::Surface::new(&context, window_arc.clone())
            {
                self.surface = Some(surface);
                self.context = Some(context);
            }
            self.window = Some(window_arc);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: RuntimeEvent) {
        match event {
            RuntimeEvent::InjectMeme {
                anchor,
                mut animation,
            } => {
                animation.reset();
                let frame = animation.current_frame();

                if let Some(ref window) = self.window {
                    self.platform_engine
                        .update_anchor(window, anchor, frame.width, frame.height)
                        .ok();
                    window.request_redraw();
                }
                self.active_animation = Some(animation);
                self.active_anchor = Some(anchor);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(animation) = &self.active_animation
            && let Some(ref window) = self.window
        {
            if let Some(anchor) = self.active_anchor {
                let frame = animation.current_frame();

                self.platform_engine
                    .update_anchor(window, anchor, frame.width, frame.height)
                    .ok();
            }

            window.request_redraw();
        }
    }
}
