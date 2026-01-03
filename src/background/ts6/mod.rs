use crate::{background::ts6::types::Ts6HidEvent, types::HidEvent};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Utf8Bytes},
};

mod auth;
mod types;

pub async fn poll_teamspeak(
    resp: mpsc::Sender<Arc<dyn HidEvent>>,
    shutting_down: Arc<AtomicBool>,
    api_key: Option<String>,
    self_name: Option<&str>,
) {
    let url = "ws://localhost:5899";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    let (mut send, mut read) = ws_stream.split();
    let ts_api_key = match api_key {
        Some(key) => key,
        None => String::new(),
    };

    let mut user_map = HashMap::new();
    let mut current_clientid = 0;

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
    send.send(Message::Text(message_text))
        .await
        .expect("Failed to send auth message");

    loop {
        if shutting_down.load(Ordering::Relaxed) {
            break;
        }

        let msg = match read.next().await {
            Some(m) => match m {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    continue;
                }
            },
            None => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };
        if let Ok(text) = msg.to_text() {
            match serde_json::from_str::<types::WsEvent>(text) {
                Ok(evt) => match evt {
                    types::WsEvent::Auth { payload, status: _ } => {
                        for user in &payload.connections[0].client_infos {
                            let user = user.clone();
                            let nick = user.properties.nickname.clone();
                            if !user_map.contains_key(&user.id) {
                                user_map.insert(user.id, nick);
                                if user.properties.nickname == self_name.unwrap_or("M1K") {
                                    current_clientid = user.id;
                                }
                            }
                        }
                    }
                    types::WsEvent::NotifyClientMoved { payload } => {
                        if !user_map.contains_key(&payload.client_id) {
                            user_map.insert(payload.client_id, payload.properties.nickname);
                        }
                    }
                    types::WsEvent::NotifyTextMessageReceived { payload } => {
                        match resp
                            .send(Arc::new(Ts6HidEvent {
                                nickname: latinrs::encode_str(&payload.invoker.nickname),
                                message: Some(latinrs::encode_str(&payload.message)),
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
                        if payload.client_id == current_clientid {
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
                                nickname: latinrs::encode_str(
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
