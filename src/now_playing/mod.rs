#[cfg(target_os = "windows")]
use core::time;
#[cfg(target_os = "windows")]
use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};

use tokio::sync::mpsc::{self};

#[cfg(target_os = "linux")]
use mpris::PlayerFinder;

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub is_shuffle: bool,
    pub artwork: Option<Vec<u8>>,
}

#[cfg(target_os = "windows")]
pub async fn poll_now_playing(resp: mpsc::Sender<MediaInfo>) -> Result<(), String> {
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
                        let mut image_vec: Option<Vec<u8>> = None;
                        match evt {
                            Model(model) => {
                                if let Some(playback) = model.playback {
                                    is_shuff = playback.shuffle;
                                }
                            }
                            Media(model, image) => {
                                if let Some(timelime) = model.timeline {
                                    if timelime.last_updated_at_ms == last_touched {
                                        continue;
                                    }
                                    last_touched = timelime.last_updated_at_ms;
                                }
                                if let Some(image) = image {
                                    image_vec = Some(image.data);
                                }
                                if let Some(media) = model.media {
                                    let mut media_info = MediaInfo {
                                        title: media.title,
                                        artist: media.artist,
                                        album: None,
                                        is_shuffle: is_shuff,
                                        artwork: image_vec,
                                    };
                                    if let Some(album) = media.album {
                                        media_info.album = Some(album.title);
                                    }
                                    resp.send(media_info).await.ok();
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

#[cfg(target_os = "linux")]
pub async fn poll_now_playing(resp: mpsc::Sender<MediaInfo>) -> Result<(), String> {
    let player = PlayerFinder::new()
        .expect("Could not connect to D-Bus")
        .find_active()
        .expect("Could not find active player");

    let events = player.events().expect("Could not start event stream");

    for event in events {
        match event {
            Ok(event) => {
                match event {
                    mpris::Event::TrackChanged(track) => {
                        let mut title = Option::None;
                        let mut artist = Option::None;
                        let mut album = Option::None;
                        let mut artwork: Option<Vec<u8>> = Option::None;

                        if let Some(t) = track.track_id() {
                            title = Some(t.to_string());
                        }

                        if let Some(a) = track.album_artists() {
                            artist = Some(a.join(", "));
                        }

                        if let Some(alb) = track.album_name() {
                            album = Some(alb.to_string());
                        }

                        if let Some(i) = track.art_url() {
                            artwork = None; //Some(i.to_string()); todo: fetch
                        }

                        let is_shuffle = player.get_shuffle().unwrap_or(false);

                        if let (Some(t), Some(a)) = (title, artist) {
                            use core::task;

                            let media_info = MediaInfo {
                                title: t,
                                artist: a,
                                album: album,
                                is_shuffle: is_shuffle,
                                artwork,
                            };
                            let resp = resp.clone();
                            tokio::task::spawn_blocking(move || {
                                resp.blocking_send(media_info).unwrap();
                            });
                            // Have to use a blocking send, as the dbus connection isnt Send
                        }
                    }
                    _ => println!("Event: {:?}", event),
                } // metadata depends on the app, so spotify provides all this, while vlc is just the filename

                //                  Metadata { values: {"xesam:album": String("One Day Remains"), "mpris:trackid": String("/com/spotify/track/4ihxQ9O2ev062rLQV9SyrN"), "mpris:artUrl": String("https://i.scdn.co/image/ab67616d0000b273bc7ddb77993dd1d8d19c22a2"), "xesam:autoRating": F64(0.0), "mpris:length": U64(260000000), "xesam:discNumber": I32(1), "xesam:trackNumber": I32(5), "xesam:url": String("https://open.spotify.com/track/4ihxQ9O2ev062rLQV9SyrN"), "xesam:albumArtist": Array([String("Alter Bridge")]), "xesam:title": String("Metalingus"), "xesam:artist": Array([String("Alter Bridge")])} }
                // Event: PlayerShutDown
            }
            Err(e) => println!("Error receiving event: {}", e),
        }
    }

    Ok(())
}
