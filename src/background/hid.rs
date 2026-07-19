use crate::nostd_types::*;
use crate::types::*;
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
    pub fn new(vid: u16, pid: u16) -> Option<Self> {
        new_device(vid, pid).map(|device| HidHandler { device, vid, pid })
    }

    pub async fn publish_hid_event(&mut self, event: Arc<dyn HidEvent>) {
        if event.event_type() == EventType::MediaUpdateShufflePlay {
            return;
        }

        for chunk in event.chunks() {
            let mut c = [0 as u8; MAX_HID_EVENT_SIZE];
            c[..chunk.len()].copy_from_slice(&chunk);
            if !self.send_to_hid_device(&c) {
                // Device is gone and could not be re-opened; drop the rest of
                // this frame rather than sending a torn event later.
                return;
            }
        }
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
