use crate::nostd_types::EventType;

pub trait HidEvent: Send + Sync {
    fn to_bytes(&self) -> Vec<u8>;
    fn chunks(&self) -> Vec<Vec<u8>>;
    fn event_type(&self) -> EventType;
    fn from_bytes(bytes: &[u8]) -> Self
    where
        Self: Sized;
}
