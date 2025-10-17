use core::time;

use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};
use tokio::sync::mpsc::{self};


#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub is_shuffle: bool,
    pub artwork: Option<Vec<u8>>,
}

pub async fn poll_now_playing(resp: mpsc::Sender<MediaInfo>) -> Result<(),String> {
    let rx = gsmtc::SessionManager::create().await;

    if let Err(err) = rx {
        return Err(err.message().to_string());
    } else {
        let mut rx = rx.unwrap();
        while let Some(evt) = rx.recv().await {
            match evt {
                SessionCreated {
                    session_id,
                    mut rx,
                    source,
                } => {
                println!("Created session: {{id={session_id}, source={source}}}");
                let mut is_shuff = false;
                let mut last_touched: i64 = 0;
                    while let Some(evt) = rx.recv().await {
                        let mut image_vec : Option<Vec<u8>> = None;
                        match evt {
                            Model(model) => {
                                if let Some(playback) = model.playback {
                                    is_shuff = playback.shuffle;
                                }
                            }
                            Media(model, image) => {
                                if let Some(timelime) = model.timeline {
                                    if timelime.last_updated_at_ms == last_touched {
                                        continue
                                    }
                                    last_touched = timelime.last_updated_at_ms;
                                }
                                if let Some(image) = image {
                                    image_vec = Some(image.data);
                                }
                                if let Some(media) = model.media{
                                    let mut media_info = MediaInfo {
                                        title: media.title,
                                        artist: media.artist,
                                        album: None,
                                        is_shuffle: is_shuff,
                                        artwork:  image_vec,
                                    };


                                    if let Some(album) = media.album {
                                        media_info.album = Some(album.title);
                                    }
                                    resp.send(media_info).await.ok();
                                }
                            },
                        }
                    }
                    println!("{source}] exited event-loop");
                }
                SessionRemoved { session_id } => println!("Session {{id={session_id}}} was removed"),
                CurrentSessionChanged {
                    session_id: Some(id),
                } => println!("Current session: {id}"),
                CurrentSessionChanged { session_id: None } => println!("No more current session"),
            }
        }  
    }
    Ok(())
}


