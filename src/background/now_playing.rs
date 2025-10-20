use crate::nostd_types::{EventType, FOOTER, HEADER};
use crate::types::HidEvent;
use image::ImageReader;
use std::{fmt::Display, io::Cursor, sync::Arc};
use tokio::sync::mpsc::{self};

#[cfg(target_os = "linux")]
use mpris::PlayerFinder;

#[cfg(target_os = "windows")]
use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub is_shuffle: Option<bool>,
    pub artwork: Option<Vec<u8>>,
}

impl Display for MediaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MediaInfo {{ title: {:?}, artist: {:?}, album: {:?}, is_shuffle: {:?}, artwork: [{} bytes] }}",
            self.title,
            self.artist,
            self.album,
            self.is_shuffle,
            match &self.artwork {
                Some(art) => art.len(),
                None => 0,
            }
        )
    }
}

impl HidEvent for MediaInfo {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if let Some(title) = &self.title {
            bytes.extend_from_slice(title.as_bytes());
        }
        bytes.push('|' as u8); // 124
        if let Some(artist) = &self.artist {
            bytes.extend_from_slice(artist.as_bytes());
        }
        bytes.push('|' as u8); // 124
        if let Some(album) = &self.album {
            bytes.extend_from_slice(album.as_bytes());
        }
        bytes.push('|' as u8); // 124
        if let Some(is_shuffle) = self.is_shuffle {
            bytes.push(if is_shuffle { 1 } else { 0 });
        }
        bytes.push('|' as u8); // 124
        if let Some(artwork) = &self.artwork {
            bytes.extend_from_slice(artwork);
        }
        bytes
    }

    fn chunks(&self) -> Vec<Vec<u8>> {
        let bytes = self.to_bytes();
        let chunk_size = 32; // Adjusted for header/footer
        let mut offset = 0;
        let mut chunks = Vec::new();
        let mut ctr = 0;
        while offset < bytes.len() {
            let mut chunk_size = chunk_size;
            let mut chunk = Vec::new();

            if ctr == 0 || offset + chunk_size >= bytes.len() {
                chunk_size -= 4; // Adjust for header/footer
            }

            if ctr == 0 {
                chunk.extend_from_slice(&HEADER); // Header
                if self.title.is_some() {
                    chunk.extend_from_slice(&[EventType::MediaUpdate as u8]);
                } else {
                    chunk.extend_from_slice(&[EventType::MediaUpdateShufflePlay as u8]);
                }
            }
            let end = std::cmp::min(offset + chunk_size, bytes.len());
            chunk.extend_from_slice(&bytes[offset..end]);

            if offset + chunk_size >= bytes.len() {
                chunk.extend_from_slice(&bytes[offset..end]);
                chunk.extend_from_slice(&FOOTER); // Footer
                chunks.push(chunk);
                break;
            }

            if chunk.len() < 32 {
                chunk.resize(32, 0);
            }
            chunks.push(chunk);
            offset += chunk_size;
            ctr += 1;
        }
        chunks
    }

    fn event_type(&self) -> crate::nostd_types::EventType {
        crate::nostd_types::EventType::MediaUpdate
    }

    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        MediaInfo {
            title: None,
            artist: None,
            album: None,
            is_shuffle: None,
            artwork: Some(bytes.to_vec()),
        }
    }
}

#[cfg(target_os = "windows")]
pub async fn poll_now_playing(resp: mpsc::Sender<Arc<dyn HidEvent>>) -> Result<(), String> {
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
                    let mut last_touched: i64 = 0;
                    let mut last_touched_s: i64 = 0;
                    while let Some(evt) = rx.recv().await {
                        let mut image_vec: Option<Vec<u8>> = None;
                        match evt {
                            Model(model) => {
                                if let Some(timelime) = model.timeline {
                                    if timelime.last_updated_at_ms == last_touched_s {
                                        continue; // we see event duplication, so account for that here.
                                    }
                                    last_touched_s = timelime.last_updated_at_ms;
                                }
                                if let Some(playback) = model.playback {
                                    let shuffle_info = MediaInfo {
                                        title: None,
                                        artist: None,
                                        album: None,
                                        is_shuffle: Some(playback.shuffle),
                                        artwork: None,
                                    };
                                    println!("{source}] Now Shuff: {shuffle_info}");
                                    resp.send(Arc::new(shuffle_info)).await.ok();
                                }
                            }
                            Media(model, image) => {
                                if let Some(timelime) = model.timeline {
                                    if timelime.last_updated_at_ms == last_touched {
                                        continue; // we see event duplication, so account for that here.
                                    }
                                    last_touched = timelime.last_updated_at_ms;
                                }
                                if let Some(image) = image {
                                    let cursor = Cursor::new(&image.data);
                                    let Ok(img_reader) =
                                        ImageReader::new(cursor).with_guessed_format()
                                    else {
                                        println!("Could not read image data");
                                        continue;
                                    };
                                    let Ok(mut image) = img_reader.decode() else {
                                        println!("Could not decode image data");
                                        continue;
                                    };

                                    image = image.thumbnail(50, 50);
                                    let e = image.to_rgba8().into_raw();
                                    println!("{}", &e.len());
                                    image_vec = Some(e);
                                }
                                if let Some(media) = model.media {
                                    let mut media_info = MediaInfo {
                                        title: Some(media.title),
                                        artist: Some(media.artist),
                                        is_shuffle: None,
                                        album: None,
                                        artwork: None, //image_vec,
                                    };
                                    if let Some(album) = media.album {
                                        media_info.album = Some(album.title);
                                    }
                                    println!("{source}] Now Playing: {media_info}");
                                    resp.send(Arc::new(media_info)).await.ok();
                                }
                            }
                        }
                    }
                    println!("{source}] exited event-loop");
                }
                SessionRemoved { session_id } => {
                    println!("Session {{id={session_id}}} was removed")
                }
                CurrentSessionChanged {
                    session_id: Some(id),
                } => println!("Current session: {id}"),
                CurrentSessionChanged { session_id: None } => println!("No more current session"),
            }
        }
    }
    Ok(())
}

//redo as yersterday
#[cfg(target_os = "linux")]
pub async fn poll_now_playing(resp: mpsc::Sender<MediaInfo>) -> Result<(), String> {
    let player = PlayerFinder::new()
        .expect("Could not connect to D-Bus")
        .find_active()
        .expect("Could not find active player");

    let events = player.events().expect("Could not start event stream");

    for event in events {
        match event {
            Ok(event) => match event {
                mpris::Event::TrackChanged(track) => {
                    let mut title = Option::None;
                    let mut artist = Option::None;
                    let mut album = Option::None;
                    let mut artwork = Option::None;

                    if let Some(t) = track.track_id() {
                        title = Some(t.to_string());
                    }

                    if let Some(a) = track.album_artists() {
                        artist = Some(a.join(", "));
                    }

                    if let Some(alb) = track.album_name() {
                        album = Some(alb.to_string());
                    }

                    let is_shuffle = player.get_shuffle().unwrap_or(false);
                    if let Some(i) = track.art_url() {
                        let resp = resp.clone();
                        let i = i.to_string();
                        tokio::task::spawn_blocking(move || {
                            if let Ok(res) = reqwest::blocking::get(i) {
                                if let Ok(bytes) = res.bytes() {
                                    let cursor = Cursor::new(&bytes);
                                    let Ok(img_reader) =
                                        ImageReader::new(cursor).with_guessed_format()
                                    else {
                                        println!("Could not read image data");
                                        return;
                                    };
                                    let Ok(mut image) = img_reader.decode() else {
                                        println!("Could not decode image data");
                                        return;
                                    };

                                    image = image.thumbnail(50, 50);
                                    let e = image.to_rgba8().into_raw();
                                    println!("{}", &e.len());
                                    artwork = Some(e);
                                }
                            }
                            artwork = artwork;
                            let media_info = MediaInfo {
                                title: title,
                                artist: artist,
                                album: album,
                                is_shuffle: Some(is_shuffle),
                                artwork,
                            };
                            let resp = resp.clone();

                            resp.blocking_send(media_info).unwrap();
                        });
                    }
                }
                _ => println!("Event: {:?}", event),
            },
            Err(e) => println!("Error receiving event: {}", e),
        }
    }

    Ok(())
}
