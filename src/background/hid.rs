use crate::nostd_types::*;
use crate::types::*;
use std::error;

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

    pub fn publish_hid_event(&self, event: impl HidEvent) {
        let bytes = event.to_bytes();
        let mut offset = 0;

        while offset < bytes.len() {
            let mut chunk: HidEventImpl = [0; MAX_HID_EVENT_SIZE];
            let mut cursor = 0;

            // Add header
            for &b in &HEADER {
                chunk[cursor] = b;
                cursor += 1;
            }

            // Add event type
            chunk[cursor] = event.event_type() as u8;
            cursor += 1;

            // Add data
            while cursor < MAX_HID_EVENT_SIZE - FOOTER.len() && offset < bytes.len() {
                chunk[cursor] = bytes[offset];
                cursor += 1;
                offset += 1;
            }

            // Add footer
            for &b in &FOOTER {
                chunk[cursor] = b;
                cursor += 1;
            }

            // Send chunk to HID device
            if let Err(e) = self.send_to_hid_device(&chunk) {
                panic!("Failed to send HID event: {}", e);
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
