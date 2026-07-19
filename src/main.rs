#![cfg(not(test))]
#![windows_subsystem = "windows"]

use lazy_static::lazy_static;
use slipstream::background::{self, hid, pc_stats::PCState, process_watcher};
use slipstream::config::Config;
use slipstream::ui::dialog::show_error_dialog;
use std::ffi::OsStr;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use sysinfo::{ProcessesToUpdate, System};
use tao::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::sync::Mutex;
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

lazy_static! {
    static ref config_res: Result<Config, String> = fs::read_to_string("config.toml")
        .map_err(|e| format!("Could not read config.toml: {e}"))
        .and_then(|contents| {
            toml::from_str::<Config>(&contents)
                .map_err(|e| format!("Could not parse config.toml: {e}"))
        });
}

#[cfg(not(test))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match config_res.as_ref() {
        Ok(config) => config,
        Err(e) => {
            let _ = show_error_dialog("neelix: Configuration Error", e);
            std::process::exit(1);
        }
    };

    let ts_api_key = config.ts6_api_key.clone();
    let self_name = config.ts6_self_name.as_deref();

    if ts_api_key.is_none() {
        let title = "neelix: Missing Teamspeak API Key";
        let message = "config.toml is missing ts6_api_key. Please add ts6_api_key to config.toml and restart the app.";
        let _ = show_error_dialog(title, message);
        std::process::exit(1);
    }

    let ts_api_key = ts_api_key.unwrap();

    let device = match config.devices.first() {
        Some(dev) => dev,
        None => {
            let _ = show_error_dialog(
                "neelix: No Devices Configured",
                "config.toml has no [[devices]] entries. Please add one and restart the app.",
            );
            std::process::exit(1);
        }
    };

    let mut hid = match hid::HidHandler::new(device.vid, device.pid).await {
        Some(hid) => hid,
        None => {
            let _ = show_error_dialog(
                "neelix: HID Device Not Found",
                &format!(
                    "Could not open HID device {:04x}:{:04x}. Is the keyboard plugged in?",
                    device.vid, device.pid
                ),
            );
            std::process::exit(1);
        }
    };

    let expected_processes = config.recognised_processes.clone();
    if expected_processes.is_empty() {
        eprintln!("Warning: No processes configured");
    }

    // Shared quit signal
    let (send_events, mut recv_events) = tokio::sync::mpsc::channel(25);

    // Atomic flag to track if we're shutting down
    let shutting_down = Arc::new(AtomicBool::new(false));

    // Spawn background tasks
    let send_events_1 = send_events.clone();
    let shutting_down_1 = shutting_down.clone();
    tokio::spawn(async move {
        background::now_playing::poll_now_playing(send_events_1, shutting_down_1).await
    });

    let send_events_2 = send_events.clone();
    let mut pc_state = PCState::init();
    let system = Arc::new(Mutex::new(System::new()));

    check_if_im_running(system.clone()).await;

    let sys_cl = system.clone();
    let shutting_down_2 = shutting_down.clone();
    tokio::spawn(async move {
        pc_state
            .poll_pc_stats(sys_cl, send_events_2, shutting_down_2)
            .await
    });

    let proc_watcher = process_watcher::ProcessWatcher::new();
    if !expected_processes.is_empty() {
        let send_events_3 = send_events.clone();
        let sys = system.clone();
        let shutting_down_3 = shutting_down.clone();
        proc_watcher.watch(sys, expected_processes, send_events_3, shutting_down_3);
    }

    // HID event handler
    let shutting_down_4 = shutting_down.clone();
    tokio::spawn(async move {
        while let Some(evt) = recv_events.recv().await {
            if shutting_down_4.load(Ordering::Relaxed) {
                break;
            }
            if !hid.publish_hid_event(evt).await {
                // Device is gone and could not be re-opened
                shutting_down_4.store(true, Ordering::Relaxed);
                break;
            }
        }
    });

    // TS6 event handler
    let shutting_down_5: Arc<AtomicBool> = shutting_down.clone();
    tokio::spawn(async move {
        background::ts6::poll_teamspeak(
            send_events,
            shutting_down_5,
            &ts_api_key,
            self_name,
            &proc_watcher,
        )
        .await
    });

    // Run tao event loop
    run_event_loop(shutting_down);

    Ok(())
}

async fn check_if_im_running(system: Arc<Mutex<System>>) {
    let mut sys = system.lock().await;
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let count = sys.processes_by_name(OsStr::new("slipstream.exe")).count();
    let count2 = sys.processes_by_name(OsStr::new("neelix.exe")).count();
    if count + count2 > 1 {
        let title = "slipstream: Already Running";
        let message = "Another instance of slipstream is already running. Please close it before starting a new one.";
        let _ = show_error_dialog(title, message);
        std::process::exit(1);
    }
}

fn run_event_loop(shutting_down: Arc<AtomicBool>) {
    let event_loop = EventLoop::new();

    // Create an invisible window (required for tray to work properly)
    let _window = WindowBuilder::new()
        .with_visible(false)
        .with_title("Slipstream QMK Daemon")
        .build(&event_loop)
        .unwrap();

    // Load the embedded icon so the binary works from any install location
    let icon_image = image::load_from_memory(include_bytes!("icon.ico"))
        .expect("Failed to decode embedded icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon_image.dimensions();
    let icon = Icon::from_rgba(icon_image.into_raw(), icon_width, icon_height)
        .expect("Failed to load icon");

    // Create tray menu
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&quit_item).unwrap();

    // Build tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Slipstream QMK Daemon")
        .with_icon(icon)
        .build()
        .unwrap();

    // Run event loop (takes 3 args: event, event_loop_window_target, control_flow)
    event_loop.run(move |event, _elwt, control_flow| {
        let quit_id = quit_item.id();

        // Check for tray menu events
        if let Ok(menu_event) = MenuEvent::receiver().try_recv() {
            if menu_event.id() == quit_id {
                shutting_down.store(true, Ordering::Relaxed);
                *control_flow = ControlFlow::Exit;
            }
        } else if shutting_down.load(Ordering::Relaxed) && *control_flow != ControlFlow::Exit {
            // A background task flagged shutdown without the user asking — surface it
            let _ = show_error_dialog(
                "Slipstream encountered an unexpected error",
                "A background task shut down unexpectedly (e.g. the HID device could not be re-opened).",
            );
            *control_flow = ControlFlow::Exit;
        } else {
            *control_flow = ControlFlow::Wait;
        }

        // Handle window events (in case window is shown later)
        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            shutting_down.store(true, Ordering::Relaxed);
            *control_flow = ControlFlow::Exit;
        }
    });
}
