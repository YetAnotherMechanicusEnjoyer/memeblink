mod app;
mod constants;
mod events;
mod ipc;
mod platform;
mod renderer;

use crate::app::MemeBlinkOverlayApp;
use crate::events::RuntimeEvent;
use crate::ipc::server::spawn_ipc_thread;
use crate::platform::wayland::WaylandOverlayEngine;
use shared::error::{MemeBlinkError, Result};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    unsafe {
        std::env::set_var("MANGOHUD", "0");
    }
    log::set_max_level(log::LevelFilter::Info);

    let event_loop = EventLoop::<RuntimeEvent>::with_user_event()
        .build()
        .map_err(|e| MemeBlinkError::WaylandInitialization(e.to_string()))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let event_proxy = event_loop.create_proxy();
    spawn_ipc_thread(event_proxy);

    let native_engine = WaylandOverlayEngine::new();

    let mut app = MemeBlinkOverlayApp::<WaylandOverlayEngine, RuntimeEvent>::new(native_engine);

    event_loop
        .run_app(&mut app)
        .map_err(|e| MemeBlinkError::WaylandInitialization(e.to_string()))
}
