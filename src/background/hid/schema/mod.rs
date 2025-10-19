#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub is_shuffle: Option<bool>,
    pub artwork: Option<Vec<u8>>,
}

pub trait HidEvent {
    fn to_bytes(&self) -> Vec<u8>;
    fn chunks(&self) -> dyn Iterator<Item = Vec<u8>>;
}


const MAX_HID_EVENT_SIZE: usize = 32;
type HidEventImpl = [u8; MAX_HID_EVENT_SIZE];

const HEADER: [u8; 4] = [0xFA, 0x00, 0xF0, 0x00];
const FOOTER: [u8; 4] = [0xAF, 0x00, 0x0F, 0x00];


enum EventType {
MediaUpdate = 0x01,
ProcessStateUpdate = 0x02,
PCUpdate = 0x03,
}
