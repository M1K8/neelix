use crate::background::{qgf_art, sanitize_hid_text};
use crate::nostd_types::SPLIT_CHAR;
use crate::nostd_types::{EventType, FOOTER, HEADER};
use crate::types::HidEvent;
use image::ImageReader;
use tokio::sync::mpsc::{self};

#[cfg(target_os = "windows")]
use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};

#[cfg(target_os = "linux")]
use mpris::PlayerFinder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{fmt::Display, io::Cursor, sync::Arc};

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub is_shuffle: Option<bool>,
    pub artwork: Option<Vec<u8>>,
    /// QGF-encoded album art built via the qmk-qgf crate; not transmitted yet.
    pub artwork_qgf: Option<Vec<u8>>,
}

impl Display for MediaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MediaInfo {{ title: {:?}, artist: {:?}, album: {:?}, is_shuffle: {:?}, artwork: [{} bytes], artwork_qgf: [{} bytes] }}",
            self.title,
            self.artist,
            self.album,
            self.is_shuffle,
            match &self.artwork {
                Some(art) => art.len(),
                None => 0,
            },
            match &self.artwork_qgf {
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
        bytes.push(SPLIT_CHAR);
        if let Some(artist) = &self.artist {
            bytes.extend_from_slice(artist.as_bytes());
        }
        bytes.push(SPLIT_CHAR);
        // if let Some(album) = &self.album {
        //     bytes.extend_from_slice(album.as_bytes());
        // }
        bytes.push(SPLIT_CHAR);
        if let Some(is_shuffle) = self.is_shuffle {
            bytes.push(if is_shuffle { 1 } else { 0 });
        }
        bytes.push(SPLIT_CHAR);
        if let Some(artwork) = &self.artwork {
            bytes.extend_from_slice(artwork);
        }
        bytes
    }

    fn chunks(&self) -> Vec<Vec<u8>> {
        let buffer = self.to_bytes();
        let chunk_size = 32;
        let mut offset = 0;
        let mut chunks = Vec::new();
        let mut header_chunk = Vec::new();
        header_chunk.extend_from_slice(&HEADER); // Header
        if self.title.is_some() {
            header_chunk.extend_from_slice(&[EventType::MediaUpdate as u8]);
        } else {
            header_chunk.extend_from_slice(&[EventType::MediaUpdateShufflePlay as u8]);
        }
        chunks.push(header_chunk);
        while offset < buffer.len() {
            let mut chunk = Vec::new();

            let end = std::cmp::min(offset + chunk_size, buffer.len());
            chunk.extend_from_slice(&buffer[offset..end]);
            if chunk.len() < 32 {
                chunk.resize(32, 0);
            }
            chunks.push(chunk);
            offset += chunk_size;
        }
        let mut footer_chunk = Vec::new();
        footer_chunk.extend_from_slice(&FOOTER);
        chunks.push(footer_chunk);

        chunks
    }

    fn event_type(&self) -> crate::nostd_types::EventType {
        crate::nostd_types::EventType::MediaUpdate
    }
}

#[cfg(target_os = "windows")]
pub async fn poll_now_playing(
    resp: mpsc::Sender<Arc<dyn HidEvent>>,
    shutting_down: Arc<AtomicBool>,
) {
    loop {
        if shutting_down.load(Ordering::Relaxed) {
            break;
        }
        let rx = gsmtc::SessionManager::create().await;
        if let Err(_) = rx {
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        } else {
            let mut rx = rx.unwrap();
            while let Some(evt) = rx.recv().await {
                match evt {
                    SessionCreated {
                        session_id,
                        mut rx,
                        source,
                    } => {
                        let mut last_touched: i64 = 0;
                        while let Some(evt) = rx.recv().await {
                            match evt {
                                Media(model, image) => {
                                    if let Some(timelime) = model.timeline {
                                        if timelime.last_updated_at_ms == last_touched {
                                            continue; // we see event duplication, so account for that here.
                                        }
                                        last_touched = timelime.last_updated_at_ms;
                                    }
                                    let mut artwork_qgf: Option<Vec<u8>> = None;
                                    if let Some(image) = image {
                                        let cursor = Cursor::new(&image.data);
                                        let Ok(img_reader) =
                                            ImageReader::new(cursor).with_guessed_format()
                                        else {
                                            //println!("Could not read image data");
                                            continue;
                                        };
                                        let Ok(mut image) = img_reader.decode() else {
                                            //println!("Could not decode image data");
                                            continue;
                                        };

                                        image = image.thumbnail(50, 50);
                                        artwork_qgf = qgf_art::image_to_qgf(&image).ok();
                                    }
                                    if let Some(media) = model.media {
                                        let mut media_info = MediaInfo {
                                            title: Some(sanitize_hid_text(&media.title)),
                                            artist: Some(sanitize_hid_text(&media.artist)),
                                            is_shuffle: None,
                                            album: None,
                                            artwork: None,
                                            artwork_qgf,
                                        };
                                        if let Some(album) = media.album {
                                            media_info.album =
                                                Some(sanitize_hid_text(&album.title));
                                        }
                                        resp.send(Arc::new(media_info)).await.ok();
                                    }
                                }
                                _ => {}
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
                    CurrentSessionChanged { session_id: None } => {
                        println!("No more current session")
                    }
                }
            }
        }
    }
}

/// Fetch album art bytes from an MPRIS art URL. These are commonly `file://`
/// URIs pointing at a local cache, so handle those without HTTP.
#[cfg(target_os = "linux")]
fn fetch_art(url: &str) -> Option<Vec<u8>> {
    if let Some(path) = url.strip_prefix("file://") {
        std::fs::read(percent_decode_path(path)).ok()
    } else {
        reqwest::blocking::get(url).ok()?.bytes().ok().map(|b| b.to_vec())
    }
}

/// Minimal percent-decoding for file:// URI paths (e.g. "%20" -> space).
#[cfg(target_os = "linux")]
fn percent_decode_path(s: &str) -> std::path::PathBuf {
    fn hex(b: u8) -> Option<u8> {
        (b as char).to_digit(16).map(|d| d as u8)
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(out).into()
}

#[cfg(target_os = "linux")]
pub async fn poll_now_playing(
    resp: mpsc::Sender<Arc<dyn HidEvent>>,
    shutting_down: Arc<AtomicBool>,
) {
    // The mpris API is blocking and its D-Bus handles are not Send, so run
    // the whole poll loop on one blocking thread instead of holding them
    // across await points.
    let _ = tokio::task::spawn_blocking(move || {
        loop {
            if shutting_down.load(Ordering::Relaxed) {
                return;
            }

            let finder = match PlayerFinder::new() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Could not connect to D-Bus: {e}");
                    std::thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };
            let player = match finder.find_active() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Could not find active player: {e}");
                    std::thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };
            let events = match player.events() {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Could not start event stream: {e}");
                    std::thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };

            for event in events {
                if shutting_down.load(Ordering::Relaxed) {
                    return;
                }
                match event {
                    Ok(mpris::Event::TrackChanged(track)) => {
                        let title = track.title().map(sanitize_hid_text);
                        let artist = track.artists().map(|a| sanitize_hid_text(&a.join(", ")));
                        let album = track.album_name().map(sanitize_hid_text);
                        let is_shuffle = player.get_shuffle().unwrap_or(false);

                        let mut artwork = None;
                        let mut artwork_qgf = None;
                        if let Some(url) = track.art_url() {
                            if let Some(bytes) = fetch_art(url) {
                                let cursor = Cursor::new(&bytes);
                                if let Ok(img_reader) =
                                    ImageReader::new(cursor).with_guessed_format()
                                {
                                    if let Ok(mut image) = img_reader.decode() {
                                        image = image.thumbnail(50, 50);
                                        artwork_qgf = qgf_art::image_to_qgf(&image).ok();
                                        artwork = Some(image.to_rgba8().into_raw());
                                    }
                                }
                            }
                        }
                        let media_info = MediaInfo {
                            title,
                            artist,
                            album,
                            is_shuffle: Some(is_shuffle),
                            artwork,
                            artwork_qgf,
                        };
                        if resp.blocking_send(Arc::new(media_info)).is_err() {
                            // Receiver gone — we're shutting down
                            return;
                        }
                    }
                    Ok(event) => println!("Event: {:?}", event),
                    Err(e) => println!("Error receiving event: {}", e),
                }
            }
            // The event stream ended (player exited) — loop around and look
            // for a new active player.
        }
    })
    .await;
}
