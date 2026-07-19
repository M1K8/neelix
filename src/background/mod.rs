pub mod clock;
pub mod hid;
pub mod now_playing;
pub mod pc_stats;
pub mod process_watcher;
pub mod qgf_art;
pub mod ts6;

/// Encode text for the HID protocol. SPLIT_CHAR is '\n', so any newline in
/// user-controlled text (song titles, nicknames, chat messages) would corrupt
/// the frame — replace them with spaces.
pub(crate) fn sanitize_hid_text(s: &str) -> String {
    latinrs::encode_str(s).replace(['\n', '\r'], " ")
}
