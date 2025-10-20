use crate::nostd_types::*;
use crate::types::*;
use std::error;
use std::sync::Arc;

pub struct HidHandler {
    device: hidapi::HidDevice,
}

impl HidHandler {
    pub fn new(vid: u16, pid: u16) -> Option<Self> {
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
                return Some(HidHandler { device });
            }
        }
        None
    }

    pub fn publish_hid_event(&self, event: Arc<dyn HidEvent>) {
        let bytes = event.chunks();
        for mut chunk in bytes {
            let mut c = [0 as u8; 32];
            c[..chunk.len()].swap_with_slice(chunk.as_mut_slice());
            if let Err(e) = self.send_to_hid_device(&c) {
                eprintln!("Error sending HID event: {}", e);
            }
        }
    }
    fn send_to_hid_device(&self, chunk: &HidEventImpl) -> Result<usize, Box<dyn error::Error>> {
        // Placeholder for actual HID device communication
        println!("Sending chunk to HID device: {:?}", chunk);
        self.device
            .write(chunk)
            .map_err(|e| Box::new(e) as Box<dyn error::Error>)
    }
}
