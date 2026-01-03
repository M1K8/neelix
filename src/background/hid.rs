use crate::nostd_types::*;
use crate::types::*;
use std::sync::Arc;

pub struct HidHandler {
    device: hidapi::HidDevice,
    vid: u16,
    pid: u16,
}
fn new_device(vid: u16, pid: u16) -> Option<hidapi::HidDevice> {
    let api = hidapi::HidApi::new().unwrap();
    let devices = api
        .device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid);
    for device in devices {
        let path = device.path();
        let device = api.open_path(path).unwrap();
        if let Err(_err) = device.write(&[0]) {
            continue;
        } else {
            return Some(device);
        }
    }
    None
}

impl HidHandler {
    pub fn new(vid: u16, pid: u16) -> Option<Self> {
        if let Some(device) = new_device(vid, pid) {
            return Some(HidHandler { device, vid, pid });
        }
        None
    }

    pub async fn publish_hid_event(&mut self, event: Arc<dyn HidEvent>) {
        if event.event_type() == EventType::MediaUpdateShufflePlay {
            return;
        }

        let bytes = event.chunks();
        for mut chunk in bytes {
            let mut c = [0 as u8; 32];
            c[..chunk.len()].swap_with_slice(chunk.as_mut_slice());
            self.send_to_hid_device(&c);
        }
    }
    fn send_to_hid_device(&mut self, chunk: &HidEventImpl) -> Option<usize> {
        // handle report ID
        let out: [u8; 33] = {
            let mut new = [0u8; 33];
            new[1..].copy_from_slice(chunk);
            new
        };

        while let Err(e) = self.device.write(&out) {
            eprintln!("Error sending to device, trying to recreate: {:?}", e);
            if let Some(d) = new_device(self.vid, self.pid) {
                self.device = d;
            } else {
                Some(1);
            }
        }
        None
    }
}
