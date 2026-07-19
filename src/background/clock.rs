use crate::nostd_types::EventType;
use crate::nostd_types::{FOOTER, HEADER};
use crate::types::HidEvent;

pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub year: u8,
    pub month: u8,
    pub day: u8,
}

impl HidEvent for Time {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.year);

        bytes.push(self.month);

        bytes.push(self.day);

        bytes.push(self.hours);

        bytes.push(self.minutes);

        bytes.push(self.seconds);

        bytes
    }

    fn chunks(&self) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        let mut header = Vec::new();
        header.extend_from_slice(&HEADER);
        header.extend_from_slice(&[EventType::Clock as u8]);
        v.push(header);

        v.push(self.to_bytes());
        let mut footer = Vec::new();
        footer.extend_from_slice(&FOOTER);
        v.push(footer);
        v
    }
    fn event_type(&self) -> EventType {
        EventType::Clock
    }
}
