#![cfg(not(test))]
#![windows_subsystem = "windows"]

use crate::background::{hid, pc_stats::PCState, process_watcher};
use crate::ui::dialog::show_error_dialog;
use lazy_static::lazy_static;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use sysinfo::System;
use tao::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::sync::Mutex;
use toml::Value;
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

extern crate hidapi;
mod background;
mod nostd_types;
mod types;
mod ui;
mod painter;

lazy_static! {
    static ref config_opt: Option<Value> = match fs::read_to_string("config.toml") {
        Ok(contents) => match toml::from_str::<toml::Value>(&contents) {
            Ok(cfg) => Some(cfg),
            Err(_e) => None,
        },
        Err(_e) => {
            None
        }
    };
}

#[cfg(not(test))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if config_opt.is_none() {
        let title = "neelix: Missing Configuration File";
        let message =
            "config.toml not found. Please create a config.toml file and restart the app.";
        let _ = show_error_dialog(title, message);
        std::process::exit(1);
    }
    let ts_api_key = config_opt.as_ref().and_then(|config| {
        config
            .get("ts6_api_key")
            .and_then(|v| v.as_str())
            .map(String::from)
    });

    let self_name = config_opt
        .as_ref()
        .and_then(|config| config.get("ts6_self_name").and_then(|v| v.as_str()));

    // If Teamspeak key is missing, show a native cross-platform dialog and exit.
    if ts_api_key.is_none() {
        let title = "neelix: Missing Teamspeak API Key";
        let message = "config.toml is missing ts6_api_key. Please add ts6_api_key to config.toml and restart the app.";
        let _ = show_error_dialog(title, message);
        std::process::exit(1);
    }

    let device = config_opt
        .as_ref()
        .and_then(|config| config.get("devices"))
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.get(0));
    let device = match device {
        Some(dev) => dev,
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No devices found in config",
            )) as Box<dyn std::error::Error>);
        }
    };

    let hid_vid = device.get("vid").unwrap().as_integer().unwrap() as u16;
    let hid_pid = device.get("pid").unwrap().as_integer().unwrap() as u16;

    let mut hid = hid::HidHandler::new(hid_vid, hid_pid).expect("Failed to initialize HID handler");

    let expected_processes: Vec<String> = config_opt
        .as_ref()
        .and_then(|config| config.get("ordered_recognised_processes"))
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if expected_processes.is_empty() {
        eprintln!("Warning: No processes configured");
    }

    // Shared quit signal
    let (send_events, mut recv_events) = tokio::sync::mpsc::channel(25);

    // Atomic flag to track if we're shutting down
    let shutting_down = Arc::new(AtomicBool::new(false));

    // Spawn background tasks
    let scl = send_events.clone();
    let sd1 = shutting_down.clone();
    tokio::spawn(async move { background::now_playing::poll_now_playing(scl, sd1).await });

    let scl2 = send_events.clone();
    let mut pc_state = PCState::init();
    let system = Arc::new(Mutex::new(System::new()));
    let sys_cl = system.clone();
    let sd2 = shutting_down.clone();
    tokio::spawn(async move { pc_state.poll_pc_stats(sys_cl, scl2, sd2).await });

    if !expected_processes.is_empty() {
        let scl3 = send_events.clone();
        let sys = system.clone();
        let sd3 = shutting_down.clone();
        tokio::spawn(async move {
            process_watcher::process_watcher(sys, expected_processes, scl3, sd3).await
        });
    }

    // HID event handler
    let sd4 = shutting_down.clone();
    tokio::spawn(async move {
        while let Some(evt) = recv_events.recv().await {
            if sd4.load(Ordering::Relaxed) {
                break;
            }
            hid.publish_hid_event(evt).await;
        }
    });

    // TS6 event handler
    let sd5: Arc<AtomicBool> = shutting_down.clone();
    tokio::spawn(async move {
        background::ts6::poll_teamspeak(send_events, sd5, ts_api_key, self_name).await
    });

    // Run tao event loop on a separate thread
    let shutting_down_clone = shutting_down.clone();

    run_event_loop(shutting_down_clone);

    Ok(())
}

fn run_event_loop(shutting_down: Arc<AtomicBool>) {
    let event_loop = EventLoop::new();

    // Create an invisible window (required for tray to work properly)
    let _window = WindowBuilder::new()
        .with_visible(false)
        .with_title("neelix")
        .build(&event_loop)
        .unwrap();

    // Load icon
    let icon = Icon::from_path(r"C:\Users\mikep\neelix\src\icon.ico", Some((64, 64)))
        .expect("Failed to load icon");

    // Create tray menu
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&quit_item).unwrap();

    // Build tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("neelix")
        .with_icon(icon)
        .build()
        .unwrap();

    // Run event loop (takes 3 args: event, event_loop_window_target, control_flow)
    event_loop.run(move |event, _elwt, control_flow| {
        let quit_id = quit_item.id();
        *control_flow = ControlFlow::Wait;

        // Check for tray menu events
        if !shutting_down.load(Ordering::Relaxed) {
            if let Ok(menu_event) = MenuEvent::receiver().try_recv() {
                if menu_event.id() == quit_id {
                    println!("Quit clicked from tray menu");
                    shutting_down.store(true, Ordering::Relaxed);
                    *control_flow = ControlFlow::Exit;
                }
            }
        }

        // Handle window events (in case window is shown later)
        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            println!("Window close requested");
            shutting_down.store(true, Ordering::Relaxed);
            *control_flow = ControlFlow::Exit;
        }
    });
}
