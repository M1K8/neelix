use crate::background::process_watcher::ProcessWatcher;
use crate::background::sanitize_hid_text;
use crate::{background::ts6::types::Ts6HidEvent, types::HidEvent};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Utf8Bytes},
};

mod types;

pub async fn poll_teamspeak(
    resp: mpsc::Sender<Arc<dyn HidEvent>>,
    shutting_down: Arc<AtomicBool>,
    api_key: &str,
    self_name: Option<&str>,
    proc_watcher: &ProcessWatcher,
) {
    let url = "ws://localhost:5899";
    let ts_api_key = api_key;

    'reconnect: loop {
        if shutting_down.load(Ordering::Relaxed) {
            break;
        }

        let ws_stream = match connect_async(url).await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("TS6: failed to connect: {e}");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        let (mut send, mut read) = ws_stream.split();

        let mut user_map = HashMap::new();
        let mut current_clientid: Option<i64> = None;

        let auth_message = json!({
            "type": "auth",
            "payload": {
                "identifier": "sh.m1k",
                "version": "0",
                "name": "neelix",
                "description": "Client for QMK Overlay",
                "content": {
                    "apiKey": ts_api_key
                }
            }
        });

        let message_text = Utf8Bytes::from(auth_message.to_string());
        if let Err(e) = send.send(Message::Text(message_text)).await {
            eprintln!("TS6: failed to send auth message: {e}");
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        }

        loop {
            if shutting_down.load(Ordering::Relaxed) {
                break 'reconnect;
            }

            if !proc_watcher.is_active("TeamSpeak.exe").await {
                // send special kill event to blank the canvas
                let kill = Ts6HidEvent {
                    nickname: "".to_string(),
                    message: None,
                    talking: false,
                    show: false,
                    is_self: false,
                };

                if let Err(e) = resp.send(Arc::new(kill)).await {
                    eprintln!("Failed to send HID event: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue 'reconnect;
            }

            let msg = match read.next().await {
                Some(Ok(msg)) => msg,
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    continue;
                }
                None => {
                    // Stream closed (TeamSpeak exited or dropped us) — reconnect.
                    eprintln!("TS6: connection closed, reconnecting");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue 'reconnect;
                }
            };
            if let Ok(text) = msg.to_text() {
                match serde_json::from_str::<types::WsEvent>(text) {
                    Ok(evt) => match evt {
                        types::WsEvent::Auth { payload, status: _ } => {
                            for connection in &payload.connections {
                                for user in &connection.client_infos {
                                    let nick = user.properties.nickname.clone();
                                    if user.properties.nickname == self_name.unwrap_or("M1K") {
                                        current_clientid = Some(user.id);
                                    }
                                    user_map.insert(user.id, nick);
                                }
                            }
                        }
                        types::WsEvent::NotifyClientMoved { payload } => {
                            user_map.insert(payload.client_id, payload.properties.nickname);
                        }
                        types::WsEvent::NotifyTextMessageReceived { payload } => {
                            match resp
                                .send(Arc::new(Ts6HidEvent {
                                    nickname: sanitize_hid_text(&payload.invoker.nickname),
                                    message: Some(sanitize_hid_text(&payload.message)),
                                    talking: false,
                                    show: true,
                                    is_self: false,
                                }))
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to send HID event: {}", e);
                                }
                            };
                            continue;
                        }
                        types::WsEvent::NotifyClientEnterView { payload } => {
                            user_map.insert(payload.client_id, payload.client_name);
                        }
                        types::WsEvent::NotifyClientLeftView { payload } => {
                            user_map.remove(&payload.client_id);
                        }
                        types::WsEvent::TalkStatusChanged { payload } => {
                            let status = if payload.status == 0 { false } else { true };
                            if Some(payload.client_id) == current_clientid {
                                let talk = Ts6HidEvent {
                                    nickname: "".to_string(),
                                    message: None,
                                    talking: status,
                                    show: true,
                                    is_self: true,
                                };

                                match resp.send(Arc::new(talk)).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("Failed to send HID event: {}", e);
                                    }
                                };
                                continue;
                            }

                            match resp
                                .send(Arc::new(Ts6HidEvent {
                                    nickname: sanitize_hid_text(
                                        &user_map
                                            .get(&payload.client_id)
                                            .cloned()
                                            .unwrap_or("Unknown".to_string()),
                                    ),
                                    message: None,
                                    talking: status,
                                    show: true,
                                    is_self: false,
                                }))
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Failed to send HID event: {}", e);
                                }
                            };
                        }
                        _ => {
                            println!("Other event: {:?}", text);
                        }
                    },
                    Err(e) => {
                        println!("Failed to parse event: {}: {}", e, text);
                    }
                }
            }
        }
    }
}
