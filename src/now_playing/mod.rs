use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};
use image::ImageReader;
use std::io::Cursor;

use neelix::schema::MediaInfo;
use tokio::sync::mpsc::{self};

#[cfg(target_os = "linux")]
use mpris::PlayerFinder;

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
                    let mut last_touched: i64 = 0;
                    while let Some(evt) = rx.recv().await {
                        let mut image_vec: Option<Vec<u8>> = None;
                        match evt {
                            Model(model) => {
                                if let Some(playback) = model.playback {
                                   let shuffle_info = MediaInfo {
                                        title: None,
                                        artist: None,
                                        album: None,
                                        is_shuffle: Some(playback.shuffle),
                                        artwork: None,
                                    };
                                    resp.send(shuffle_info).await.ok();
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
                                        artwork: None// image_vec,
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

/*
pub async fn poll_now_playing_l(resp: mpsc::Sender<MediaInfo>) -> Result<(), String> {
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
                            if let Ok(res) = reqwest::get(i).await {
                                if let Ok(bytes) = res.bytes().await {
                                    // use ispc_downsampler::Image;

                                    // ispc_downsampler::downsample(Image::new(pixels, width, height, format), 60, 60)
                                }
                            }
                            artwork = None; //Some(i.to_string()); todo: fetch
                        }

                        let is_shuffle = player.get_shuffle().unwrap_or(false);

                        if let (Some(t), Some(a)) = (title, artist) {
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

*/
