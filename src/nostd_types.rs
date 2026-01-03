#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EventType {
    None = 0x0,
    MediaUpdate = 0x01,
    MediaUpdateShufflePlay = 0x02,
    ProcessStateUpdate = 0x03,
    PCUpdate = 0x04,
    RawString = 0x05,
    TS6 = 0x06,
}

impl EventType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => EventType::MediaUpdate,
            0x02 => EventType::MediaUpdateShufflePlay,
            0x03 => EventType::ProcessStateUpdate,
            0x04 => EventType::PCUpdate,
            0x05 => EventType::RawString,
            0x06 => EventType::TS6,
            _ => EventType::None,
        }
    }
}

pub struct ScreenSpace {
    pub x: u16,
    pub y: u16,
    pub x2: u16,
    pub y2: u16,
}

pub const MAX_HID_EVENT_SIZE: usize = 32;
pub type HidEventImpl = [u8; MAX_HID_EVENT_SIZE];

pub const HEADER: [u8; 3] = [0xFA, 0x00, 0xF0];
pub const FOOTER: [u8; 4] = [0xAF, 0x00, 0x0F, 0x00];
pub const TYPE_BIT: usize = 3;
pub const MUSIC: ScreenSpace = ScreenSpace {
    x: 0,
    y: 0,
    x2: 300,
    y2: 80,
};

pub const CPU: ScreenSpace = ScreenSpace {
    x: 305,
    y: 0,
    x2: 310,
    y2: 240,
};
pub const RAM: ScreenSpace = ScreenSpace {
    x: 312,
    y: 0,
    x2: 317,
    y2: 240,
};

pub const TSSELF: ScreenSpace = ScreenSpace {
    x: 0,
    y: 83,
    x2: 270,
    y2: 110,
};

pub const TS: ScreenSpace = ScreenSpace {
    x: 0,
    y: 110,
    x2: 300,
    y2: 220,
};

pub const TS_BUBBLE: ScreenSpace = ScreenSpace {
    x: 275,
    y: 229,
    x2: 285,
    y2: 240,
};
