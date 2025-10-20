pub enum EventType {
    None = 0x0,
    MediaUpdate = 0x01,
    MediaUpdateShufflePlay = 0x02,
    ProcessStateUpdate = 0x03,
    PCUpdate = 0x04,
    RawString = 0x05,
}

impl EventType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => EventType::MediaUpdate,
            0x02 => EventType::MediaUpdateShufflePlay,
            0x03 => EventType::ProcessStateUpdate,
            0x04 => EventType::PCUpdate,
            0x05 => EventType::RawString,
            _ => EventType::None,
        }
    }
}

pub const MAX_HID_EVENT_SIZE: usize = 32;
pub type HidEventImpl = [u8; MAX_HID_EVENT_SIZE];

pub const HEADER: [u8; 3] = [0xFA, 0x00, 0xF0];
pub const FOOTER: [u8; 4] = [0xAF, 0x00, 0x0F, 0x00];
pub const TYPE_BIT: usize = 3;
