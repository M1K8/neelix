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

// todo - implement state, internal q, default chunking over bytes, impl event for media to start woth