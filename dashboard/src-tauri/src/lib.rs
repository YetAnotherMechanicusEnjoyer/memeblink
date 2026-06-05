#![recursion_limit = "1024"]
use shared::constants::IPC_SOCKET_ADDRESS;
use shared::models::OverlayEvent;
use std::io::Write;
use std::os::unix::net::UnixStream;
use tauri::command;
use tokio::sync::Mutex;

use crate::matrix::MatrixSession;

mod matrix;

#[command]
fn inject_meme(event: OverlayEvent) -> Result<(), String> {
    let mut payload = serde_json::to_string(&event).map_err(|e| e.to_string())?;
    payload.push('\n');

    let mut stream = UnixStream::connect(IPC_SOCKET_ADDRESS)
        .map_err(|e| format!("Impossible to connect to overlay: {e}"))?;

    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("Error writing IPC: {e}"))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(MatrixSession {
            client: Mutex::new(None),
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            inject_meme,
            matrix::send_meme_to_matrix,
            matrix::start_matrix_sso_auth,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
