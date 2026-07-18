//! Builds QGF (Quantum Graphics Format) representations of images used by the
//! background tasks, via the `qmk-qgf` crate (linked as `qgf`).
//!
//! Nothing consumes these representations yet — they are built alongside the
//! existing artwork/process handling so they can be wired up to the HID
//! transport later.

use image::{DynamicImage, ImageReader};
use std::io::Cursor;
use std::path::Path;

/// Format used when encoding QGF output. Rgb565 matches what the painter
/// examples target; swap this if the display wants a palette format.
const QGF_FORMAT: qgf::QgfFormat = qgf::QgfFormat::Rgb565;

/// Encode an already-decoded image as a single-frame QGF.
pub fn image_to_qgf(image: &DynamicImage) -> Result<Vec<u8>, qgf::QgfError> {
    let rgb = image.to_rgb8();
    let frame = qgf::Frame {
        width: rgb.width(),
        height: rgb.height(),
        rgb: rgb.into_raw(),
        delay_ms: None,
    };
    qgf::encode(&[frame], QGF_FORMAT, &qgf::EncodeOptions::default())
}

/// Decode raw image bytes (.ico, .png, .jpg — anything the `image` crate
/// recognises) and encode them as QGF.
pub fn image_bytes_to_qgf(data: &[u8]) -> Option<Vec<u8>> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let image = reader.decode().ok()?;
    image_to_qgf(&image).ok()
}

/// Build the QGF representation of the .ico associated with a recognised
/// process. The icon is looked up as `icons/<process stem>.ico` relative to
/// the working directory (the same place `config.toml` is loaded from), e.g.
/// `steam.exe` -> `icons/steam.ico`. Returns None when no icon exists or it
/// cannot be decoded.
pub fn process_icon_qgf(process_name: &str) -> Option<Vec<u8>> {
    let stem = Path::new(process_name).file_stem()?.to_str()?;
    let data = std::fs::read(Path::new("icons").join(format!("{stem}.ico"))).ok()?;
    image_bytes_to_qgf(&data)
}
