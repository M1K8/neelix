pub enum EventType {
    MediaUpdate = 0x01,
    ProcessStateUpdate = 0x02,
    PCUpdate = 0x03,
}

pub const MAX_HID_EVENT_SIZE: usize = 32;
pub type HidEventImpl = [u8; MAX_HID_EVENT_SIZE];

pub const HEADER: [u8; 3] = [0xFA, 0x00, 0xF0];
pub const FOOTER: [u8; 3] = [0xAF, 0x0F, 0x00];
