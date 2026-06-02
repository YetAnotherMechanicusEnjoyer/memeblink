use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

use crate::constants::{IPC_SOCKET_ADDRESS, MAX_IPC_PAYLOAD_BYTES};
use crate::events::RuntimeEvent;
use crate::renderer::decoder::fetch_and_decode_asset;
use shared::error::{MemeBlinkError, Result};
use shared::models::OverlayEvent;
use winit::event_loop::EventLoopProxy;

#[inline]
pub fn spawn_ipc_thread(proxy: EventLoopProxy<RuntimeEvent>) {
    thread::spawn(move || {
        if let Err(e) = run_listener_loop(proxy) {
            eprintln!("[MemeBlink IPC Error] Fatal crash: {}", e);
            log::error!("IPC Server encountered a fatal error: {}", e);
        }
    });
}

fn run_listener_loop(proxy: EventLoopProxy<RuntimeEvent>) -> Result<()> {
    if std::fs::metadata(IPC_SOCKET_ADDRESS).is_ok() {
        let _ = std::fs::remove_file(IPC_SOCKET_ADDRESS);
    }

    let listener =
        UnixListener::bind(IPC_SOCKET_ADDRESS).map_err(|e| MemeBlinkError::IpcBinding {
            path: IPC_SOCKET_ADDRESS.to_string(),
            source: e,
        })?;

    println!(
        "[MemeBlink IPC] Server listening on socket: {}",
        IPC_SOCKET_ADDRESS
    );
    log::info!("IPC Server listening on filesystem socket.");

    for stream in listener.incoming() {
        match stream {
            Ok(client) => handle_client(client, &proxy),
            Err(e) => log::warn!("Failed to accept IPC connection: {}", e),
        }
    }

    Ok(())
}

fn handle_client(client: UnixStream, proxy: &EventLoopProxy<RuntimeEvent>) {
    let mut reader = BufReader::new(client);
    let mut buffer = String::with_capacity(MAX_IPC_PAYLOAD_BYTES);

    if let Ok(bytes_read) = reader.read_line(&mut buffer)
        && bytes_read > 0
        && bytes_read <= MAX_IPC_PAYLOAD_BYTES
        && let Ok(event) = serde_json::from_str::<OverlayEvent>(&buffer)
    {
        if let Ok(asset) = fetch_and_decode_asset(&event.image_path, event.width, event.height) {
            let runtime_event = RuntimeEvent::InjectMeme {
                anchor: event.anchor,
                asset,
                duration: std::time::Duration::from_millis(event.duration_ms as u64),
                custom_x: event.x,
                custom_y: event.y,
            };
            let _ = proxy.send_event(runtime_event);
        } else {
            eprintln!(
                "[MemeBlink IPC] Failed to decode image asset: {}",
                event.image_path
            );
            log::error!("Failed to decode image asset: {}", event.image_path);
        }
    }
}
