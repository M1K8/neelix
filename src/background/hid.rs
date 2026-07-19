use crate::background::clock;
use crate::nostd_types::*;
use crate::types::*;
use chrono::{Datelike, Timelike};
use std::sync::Arc;

pub struct HidHandler {
    device: hidapi::HidDevice,
    vid: u16,
    pid: u16,
}
fn new_device(vid: u16, pid: u16) -> Option<hidapi::HidDevice> {
    let api = match hidapi::HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            eprintln!("Failed to initialise hidapi: {e}");
            return None;
        }
    };
    let devices = api
        .device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid);
    for device in devices {
        let Ok(device) = api.open_path(device.path()) else {
            continue;
        };
        if device.write(&[0]).is_ok() {
            return Some(device);
        }
    }
    None
}

impl HidHandler {
    pub async fn new(vid: u16, pid: u16) -> Option<Self> {
        let mut h = new_device(vid, pid).map(|device| HidHandler { device, vid, pid })?;
        let now = chrono::offset::Local::now();
        println!(
            "Publishing initial clock event: {:?}:{:?}:{:?} {:?}:{:?}:{:?}",
            now.hour(),
            now.minute(),
            now.second(),
            now.year(),
            now.month(),
            now.day()
        );
        h.publish_hid_event(Arc::new(clock::Time {
            hours: now.hour() as u8,
            minutes: now.minute() as u8,
            seconds: now.second() as u8,
            year: (now.year() - 1900) as u8,
            month: (now.month() - 1u32) as u8, // this is actually zero indexed: https://github.com/ChibiOS/ChibiOS/blob/259505e28665781f23323020174302cfa73fd48d/os/hal/src/hal_rtc.c#L265
            day: now.day() as u8,
        }))
        .await;
        Some(h)
    }

    /// Returns false when the device is gone and could not be re-opened.
    pub async fn publish_hid_event(&mut self, event: Arc<dyn HidEvent>) -> bool {
        if event.event_type() == EventType::MediaUpdateShufflePlay {
            return true;
        }

        for chunk in event.chunks() {
            let mut c = [0 as u8; MAX_HID_EVENT_SIZE];
            c[..chunk.len()].copy_from_slice(&chunk);
            if !self.send_to_hid_device(&c) {
                // Device is gone and could not be re-opened; drop the rest of
                // this frame rather than sending a torn event later.
                return false;
            }
        }
        true
    }
    fn send_to_hid_device(&mut self, chunk: &HidEventImpl) -> bool {
        // handle report ID
        let out: [u8; 33] = {
            let mut new = [0u8; 33];
            new[1..].copy_from_slice(chunk);
            new
        };

        for _ in 0..3 {
            match self.device.write(&out) {
                Ok(_) => return true,
                Err(e) => {
                    eprintln!("Error sending to device, trying to recreate: {:?}", e);
                    match new_device(self.vid, self.pid) {
                        Some(d) => self.device = d,
                        None => return false,
                    }
                }
            }
        }
        false
    }
}
