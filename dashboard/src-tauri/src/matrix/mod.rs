pub mod parser;

use matrix_sdk::{
    ruma::events::room::{
        message::{MessageType, OriginalSyncRoomMessageEvent},
        MediaSource,
    },
    Client, Room,
};
use parser::parse_matrix_message;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::Mutex;

pub struct MatrixSession {
    pub client: Mutex<Option<Client>>,
}

#[tauri::command]
pub async fn start_matrix_sso_auth(
    app_handle: AppHandle,
    homeserver_url: String,
    room_id_str: String,
    session: State<'_, MatrixSession>,
) -> Result<String, String> {
    let client = Client::builder()
        .homeserver_url(&homeserver_url)
        .build()
        .await
        .map_err(|e| format!("Client error: {}", e))?;

    let app_clone = app_handle.clone();

    let response = client
        .matrix_auth()
        .login_sso(|sso_url| async move {
            app_clone
                .opener()
                .open_url(&sso_url, None::<&str>)
                .map_err(|e| matrix_sdk::Error::UnknownError(e.to_string().into()))?;
            Ok(())
        })
        .initial_device_display_name("MemeBlink Overlay")
        .await
        .map_err(|e| format!("SSO failure: {}", e))?;

    let user_id = response.user_id.to_string();

    let room_id = <&matrix_sdk::ruma::RoomId>::try_from(room_id_str.as_str())
        .map_err(|_| "Invalid Matrix Room ID format")?;

    client
        .join_room_by_id(room_id)
        .await
        .map_err(|e| format!("Could not join the room: {}", e))?;

    client.add_event_handler(move |event: OriginalSyncRoomMessageEvent, _room: Room| {
        let app = app_handle.clone();
        async move {
            match event.content.msgtype {
                MessageType::Image(image_content) => {
                    let mxc_string = match &image_content.source {
                        MediaSource::Plain(url) => url.to_string(),
                        MediaSource::Encrypted(file) => file.url.to_string(),
                    };
                    let http_image_url = format!(
                        "https://matrix.org/_matrix/media/v3/download/{}",
                        mxc_string.trim_start_matches("mxc://")
                    );
                    let config = parse_matrix_message(&image_content.body);

                    let payload = serde_json::json!({
                        "image_path": http_image_url,
                        "text": config.text,
                        "duration_ms": config.duration_ms,
                        "width": config.width,
                        "height": config.height,
                        "text_color": config.text_color,
                        "sender": event.sender.to_string(),
                    });
                    let _ = app.emit("matrix_meme_received", &payload);
                }
                MessageType::Text(text_content) => {
                    let config = parse_matrix_message(&text_content.body);
                    if config.text.is_some() {
                        let image_path = config.image_url.unwrap_or_else(|| {
                            if let Some(pos) = text_content.body.find("http") {
                                text_content.body[pos..].trim().to_string()
                            } else {
                                "".to_string()
                            }
                        });
                        let payload = serde_json::json!({
                            "image_path": image_path,
                            "text": config.text,
                            "duration_ms": config.duration_ms,
                            "width": config.width,
                            "height": config.height,
                            "text_color": config.text_color,
                            "sender": event.sender.to_string(),
                        });
                        let _ = app.emit("matrix_meme_received", &payload);
                    }
                }
                _ => {}
            }
        }
    });

    let mut guard = session.client.lock().await;
    *guard = Some(client.clone());

    tokio::spawn(async move {
        let sync_settings = matrix_sdk::config::SyncSettings::default();
        if let Err(why) = client.sync(sync_settings).await {
            eprintln!("Matrix synchronization error: {:?}", why);
        }
    });

    Ok(format!(
        "Connected as {} and listening to {}",
        user_id, room_id_str
    ))
}

#[tauri::command]
pub async fn send_meme_to_matrix(
    room_id_str: String,
    text: String,
    image_url: String,
    duration_ms: u32,
    text_color: String,
    state: tauri::State<'_, MatrixSession>,
) -> Result<(), String> {
    let guard = state.client.lock().await;
    let client = guard
        .as_ref()
        .ok_or_else(|| "You must connect to Matrix first!".to_string())?;

    let room_id = <&matrix_sdk::ruma::RoomId>::try_from(room_id_str.as_str())
        .map_err(|_| "Invalid Room ID format".to_string())?;

    let room = client
        .get_room(room_id)
        .ok_or_else(|| "The bot could not find or has not joined this room.".to_string())?;

    let message_body = format!(
        "{} -d {} -c {} -i {}",
        text, duration_ms, text_color, image_url
    );

    if let matrix_sdk::RoomState::Joined = room.state() {
        use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
        let content = RoomMessageEventContent::text_plain(message_body);
        room.send(content)
            .await
            .map_err(|e| format!("Error sending message to Matrix: {}", e))?;
    } else {
        return Err("The bot is not in this room (status is not joined).".to_string());
    }

    Ok(())
}
