use std::{collections::HashSet, fs};

use crate::background::process_watcher;

extern crate hidapi;
mod background;
mod nostd_types;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_contents = fs::read_to_string("config.toml")?;
    let config: toml::Value = toml::from_str(&config_contents)?;

    println!("Config: {:?}", config);

    let expected_processes = if let Some(processes) = config.get("recognised_processes") {
        processes
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect()
    } else {
        println!("non");
        HashSet::new()
    };

    let (send, recv) = tokio::sync::mpsc::channel(100);
    let (send_media, mut recv_media) = tokio::sync::mpsc::channel(5);

    tokio::spawn(async move { background::now_playing::poll_now_playing(send_media).await });

    while let Some(evt) = recv_media.recv().await {
        println!("Now Playing: {:?}", evt);
    }

    if expected_processes.is_empty() {
        eprintln!("No recognised_processes found in config.toml");
    } else {
        if let Err(err) =
            tokio::spawn(process_watcher::process_watcher(expected_processes, send)).await
        {
            eprintln!("Error spawning process watcher: {}", err);
        }
    }

    Ok(())
}

// for known vid / pid pairs, either config known usage page / usage, or try all
