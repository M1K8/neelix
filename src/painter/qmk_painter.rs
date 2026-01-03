//! QMK Painter module - Rust implementation of Python painter.py and painter_qgf.py
//! 
//! This module provides image format conversion, QMK RLE compression, and QGF file generation
//! compatible with QMK keyboard firmware's Quantum Painter graphics system.

use std::io::{self};

/// Supported image formats for Quantum Painter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Rgb888,
    Rgb565,
    Pal256,
    Pal16,
    Pal4,
    Pal2,
    Mono256,
    Mono16,
    Mono4,
    Mono2,
}

/// Metadata for an image format
#[derive(Debug, Clone)]
pub struct FormatMetadata {
    pub image_format: &'static str,
    pub bpp: u32,
    pub has_palette: bool,
    pub num_colors: u32,
    pub image_format_byte: u8,
}

impl ImageFormat {
    /// Get metadata for this format
    pub fn metadata(&self) -> FormatMetadata {
        match self {
            ImageFormat::Rgb888 => FormatMetadata {
                image_format: "IMAGE_FORMAT_RGB888",
                bpp: 24,
                has_palette: false,
                num_colors: 16777216,
                image_format_byte: 0x09,
            },
            ImageFormat::Rgb565 => FormatMetadata {
                image_format: "IMAGE_FORMAT_RGB565",
                bpp: 16,
                has_palette: false,
                num_colors: 65536,
                image_format_byte: 0x08,
            },
            ImageFormat::Pal256 => FormatMetadata {
                image_format: "IMAGE_FORMAT_PALETTE",
                bpp: 8,
                has_palette: true,
                num_colors: 256,
                image_format_byte: 0x07,
            },
            ImageFormat::Pal16 => FormatMetadata {
                image_format: "IMAGE_FORMAT_PALETTE",
                bpp: 4,
                has_palette: true,
                num_colors: 16,
                image_format_byte: 0x06,
            },
            ImageFormat::Pal4 => FormatMetadata {
                image_format: "IMAGE_FORMAT_PALETTE",
                bpp: 2,
                has_palette: true,
                num_colors: 4,
                image_format_byte: 0x05,
            },
            ImageFormat::Pal2 => FormatMetadata {
                image_format: "IMAGE_FORMAT_PALETTE",
                bpp: 1,
                has_palette: true,
                num_colors: 2,
                image_format_byte: 0x04,
            },
            ImageFormat::Mono256 => FormatMetadata {
                image_format: "IMAGE_FORMAT_GRAYSCALE",
                bpp: 8,
                has_palette: false,
                num_colors: 256,
                image_format_byte: 0x03,
            },
            ImageFormat::Mono16 => FormatMetadata {
                image_format: "IMAGE_FORMAT_GRAYSCALE",
                bpp: 4,
                has_palette: false,
                num_colors: 16,
                image_format_byte: 0x02,
            },
            ImageFormat::Mono4 => FormatMetadata {
                image_format: "IMAGE_FORMAT_GRAYSCALE",
                bpp: 2,
                has_palette: false,
                num_colors: 4,
                image_format_byte: 0x01,
            },
            ImageFormat::Mono2 => FormatMetadata {
                image_format: "IMAGE_FORMAT_GRAYSCALE",
                bpp: 1,
                has_palette: false,
                num_colors: 2,
                image_format_byte: 0x00,
            },
        }
    }
}

/// Represents an RGB color
#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RgbColor { r, g, b }
    }

    /// Convert RGB to HSV in QMK's 0-255 scale
    pub fn to_qmk_hsv(&self) -> (u8, u8, u8) {
        let rf = self.r as f64 / 255.0;
        let gf = self.g as f64 / 255.0;
        let bf = self.b as f64 / 255.0;

        let max = rf.max(gf).max(bf);
        let min = rf.min(gf).min(bf);
        let delta = max - min;

        // Value
        let v = (max * 255.0) as u8;

        // Saturation
        let s = if max == 0.0 {
            0u8
        } else {
            (delta / max * 255.0) as u8
        };

        // Hue
        let h = if delta == 0.0 {
            0u8
        } else if max == rf {
            (((gf - bf) / delta) % 6.0 / 6.0 * 255.0) as u8
        } else if max == gf {
            (((bf - rf) / delta + 2.0) / 6.0 * 255.0) as u8
        } else {
            (((rf - gf) / delta + 4.0) / 6.0 * 255.0) as u8
        };

        (h, s as u8, v)
    }
}

/// Rescales a byte value from [0,255] to [0,maxval]
pub fn rescale_byte(val: u8, maxval: u32) -> u8 {
    ((val as u32 * maxval) / 255) as u8
}

/// Convert RGB888 to RGB565 (5-6-5 format)
pub fn rgb_to565(r: u8, g: u8, b: u8) -> [u8; 2] {
    let msb = (((r >> 3) & 0x1F) << 3) + ((g >> 5) & 0x07);
    let lsb = (((g >> 2) & 0x07) << 5) + ((b >> 3) & 0x1F);
    [msb as u8, lsb as u8]
}

/// Represents converted pixel data with optional palette
#[derive(Debug, Clone)]
pub struct ConvertedImageData {
    pub palette: Option<Vec<RgbColor>>,
    pub pixel_data: Vec<u8>,
}

/// Converts raw pixel bytes according to the specified format
pub fn convert_image_bytes(
    pixels: &[u8],
    width: u32,
    height: u32,
    format: &FormatMetadata,
) -> Result<ConvertedImageData, String> {
    let ncolors = format.num_colors;
    let bpp = format.bpp as f64;
    let shifter = (ncolors as f64).log2() as u32;
    let pixels_per_byte = (8.0 / bpp) as u32;
    let bytes_per_pixel = ((bpp.ceil()) / 8.0).ceil() as u32;

    let expected_byte_count = if pixels_per_byte != 0 {
        ((width * height) + (pixels_per_byte - 1)) / pixels_per_byte
    } else {
        width * height * bytes_per_pixel
    };

    let mut pixel_data = Vec::with_capacity(expected_byte_count as usize);

    match format.image_format {
        "IMAGE_FORMAT_GRAYSCALE" => {
            // Grayscale: pack pixel values
            for x in 0..expected_byte_count {
                let mut byte = 0u8;
                for n in 0..pixels_per_byte {
                    let byte_offset = (x * pixels_per_byte + n) as usize;
                    if byte_offset < pixels.len() {
                        let pixel_val = rescale_byte(pixels[byte_offset], ncolors - 1);
                        byte |= pixel_val << (n * shifter);
                    }
                }
                pixel_data.push(byte);
            }
        }
        "IMAGE_FORMAT_PALETTE" => {
            // Palette: pack palette indices
            for x in 0..expected_byte_count {
                let mut byte = 0u8;
                for n in 0..pixels_per_byte {
                    let byte_offset = (x * pixels_per_byte + n) as usize;
                    if byte_offset < pixels.len() {
                        byte |= (pixels[byte_offset] & (ncolors as u8 - 1)) << (n * shifter);
                    }
                }
                pixel_data.push(byte);
            }
        }
        "IMAGE_FORMAT_RGB565" => {
            // RGB565: process R, G, B channels
            if pixels.len() % 3 != 0 {
                return Err("RGB565 requires RGB triplets".to_string());
            }
            for i in (0..pixels.len()).step_by(3) {
                let [msb, lsb] = rgb_to565(pixels[i], pixels[i + 1], pixels[i + 2]);
                pixel_data.push(msb);
                pixel_data.push(lsb);
            }
        }
        "IMAGE_FORMAT_RGB888" => {
            // RGB888: direct copy
            pixel_data.extend_from_slice(pixels);
        }
        _ => return Err(format!("Unknown format: {}", format.image_format)),
    }

    if pixel_data.len() != expected_byte_count as usize {
        return Err(format!(
            "Wrong byte count: got {}, expected {}",
            pixel_data.len(),
            expected_byte_count
        ));
    }

    Ok(ConvertedImageData {
        palette: None,
        pixel_data,
    })
}

/// QMK RLE compression
///
/// Two compression modes:
/// - Non-repeating: marker >= 128, length = marker - 128, followed by that many data bytes
/// - Repeating: marker < 128, length = marker, followed by single byte to repeat
#[derive(Debug)]
pub struct RleCompressor;

impl RleCompressor {
    /// Compress bytes using QMK RLE algorithm
    pub fn compress(data: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        let mut temp = Vec::new();
        let mut repeat = false;

        for (n, &c) in data.iter().enumerate() {
            let end = n == data.len() - 1;

            temp.push(c);

            if temp.len() <= 1 {
                continue;
            }

            if repeat {
                if temp[temp.len() - 1] != temp[temp.len() - 2] {
                    repeat = false;
                }
            }

            if !repeat || temp.len() == 128 || end {
                let len = if end { temp.len() } else { temp.len() - 1 };
                output.push(len as u8);
                output.push(temp[0]);
                temp = vec![temp[temp.len() - 1]];
                repeat = false;
            } else if temp.len() >= 2 && temp[temp.len() - 1] == temp[temp.len() - 2] {
                repeat = true;
                if temp.len() > 2 {
                    output.push((temp.len() - 2 + 128) as u8);
                    output.extend_from_slice(&temp[0..(temp.len() - 2)]);
                    temp = vec![temp[temp.len() - 1], temp[temp.len() - 1]];
                }
                continue;
            }

            if temp.len() == 128 || end {
                output.push((temp.len() + 128) as u8);
                output.extend_from_slice(&temp);
                temp.clear();
                repeat = false;
            }
        }

        output
    }

    /// Decompress QMK RLE data
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut output = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let marker = data[i];
            i += 1;

            if marker >= 128 {
                // Non-repeating block
                let length = (marker - 128) as usize;
                if i + length > data.len() {
                    return Err("Incomplete non-repeating block".to_string());
                }
                output.extend_from_slice(&data[i..i + length]);
                i += length;
            } else {
                // Repeating block
                let length = marker as usize;
                if i >= data.len() {
                    return Err("Missing repeat byte".to_string());
                }
                let repeat_byte = data[i];
                i += 1;
                output.resize(output.len() + length, repeat_byte);
            }
        }

        Ok(output)
    }
}

/// QGF File format structures and writing
pub mod qgf {
    use super::*;
    use std::io::Write;

    pub const QGF_MAGIC: u32 = 0x464751; // "QGF" in little-endian
    pub const QGF_VERSION: u8 = 1;

    /// Block type identifiers
    #[derive(Debug, Copy, Clone)]
    pub enum BlockType {
        GraphicsDescriptor = 0x00,
        FrameOffsetDescriptor = 0x01,
        FrameDescriptor = 0x02,
        FramePaletteDescriptor = 0x03,
        FrameDeltaDescriptor = 0x04,
        FrameDataDescriptor = 0x05,
    }

    /// Write little-endian 16-bit value
    fn write_u16_le(writer: &mut dyn Write, val: u16) -> io::Result<()> {
        writer.write_all(&val.to_le_bytes())
    }

    /// Write little-endian 32-bit value
    fn write_u32_le(writer: &mut dyn Write, val: u32) -> io::Result<()> {
        writer.write_all(&val.to_le_bytes())
    }

    /// Write 24-bit little-endian value
    fn write_u24_le(writer: &mut dyn Write, val: u32) -> io::Result<()> {
        let bytes = [
            (val & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            ((val >> 16) & 0xFF) as u8,
        ];
        writer.write_all(&bytes)
    }

    /// QGF Block header (5 bytes)
    pub struct BlockHeader {
        pub type_id: u8,
        pub length: u32,
    }

    impl BlockHeader {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            writer.write_all(&[self.type_id])?;
            writer.write_all(&[!self.type_id])?;
            write_u24_le(writer, self.length)?;
            Ok(())
        }
    }

    /// Graphics descriptor block
    pub struct GraphicsDescriptor {
        pub version: u8,
        pub total_file_size: u32,
        pub image_width: u16,
        pub image_height: u16,
        pub frame_count: u16,
    }

    impl GraphicsDescriptor {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            let mut header = BlockHeader {
                type_id: BlockType::GraphicsDescriptor as u8,
                length: 18,
            };
            header.write(writer)?;

            write_u24_le(writer, QGF_MAGIC)?;
            writer.write_all(&[self.version])?;
            write_u32_le(writer, self.total_file_size)?;
            write_u32_le(writer, !self.total_file_size)?;
            write_u16_le(writer, self.image_width)?;
            write_u16_le(writer, self.image_height)?;
            write_u16_le(writer, self.frame_count)?;

            Ok(())
        }

        pub fn size() -> usize {
            5 + 18 // header + content
        }
    }

    /// Frame descriptor (per-frame metadata)
    pub struct FrameDescriptor {
        pub format: u8,
        pub flags: u8,
        pub compression: u8,
        pub transparency_index: u8,
        pub delay: u16,
    }

    impl FrameDescriptor {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            let header = BlockHeader {
                type_id: BlockType::FrameDescriptor as u8,
                length: 6,
            };
            header.write(writer)?;

            writer.write_all(&[self.format])?;
            writer.write_all(&[self.flags])?;
            writer.write_all(&[self.compression])?;
            writer.write_all(&[self.transparency_index])?;
            write_u16_le(writer, self.delay)?;

            Ok(())
        }

        pub fn size() -> usize {
            5 + 6 // header + content
        }

        pub fn is_transparent(&self) -> bool {
            (self.flags & 0x01) != 0
        }

        pub fn set_transparent(&mut self, transparent: bool) {
            if transparent {
                self.flags |= 0x01;
            } else {
                self.flags &= !0x01;
            }
        }

        pub fn is_delta(&self) -> bool {
            (self.flags & 0x02) != 0
        }

        pub fn set_delta(&mut self, delta: bool) {
            if delta {
                self.flags |= 0x02;
            } else {
                self.flags &= !0x02;
            }
        }
    }

    /// Palette descriptor for indexed color formats
    pub struct PaletteDescriptor {
        pub entries: Vec<(u8, u8, u8)>, // (H, S, V)
    }

    impl PaletteDescriptor {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            let header = BlockHeader {
                type_id: BlockType::FramePaletteDescriptor as u8,
                length: (self.entries.len() * 3) as u32,
            };
            header.write(writer)?;

            for (h, s, v) in &self.entries {
                writer.write_all(&[*h])?;
                writer.write_all(&[*s])?;
                writer.write_all(&[*v])?;
            }

            Ok(())
        }
    }

    /// Delta descriptor for delta frames
    pub struct DeltaDescriptor {
        pub left: u16,
        pub top: u16,
        pub right: u16,
        pub bottom: u16,
    }

    impl DeltaDescriptor {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            let header = BlockHeader {
                type_id: BlockType::FrameDeltaDescriptor as u8,
                length: 8,
            };
            header.write(writer)?;

            write_u16_le(writer, self.left)?;
            write_u16_le(writer, self.top)?;
            write_u16_le(writer, self.right)?;
            write_u16_le(writer, self.bottom)?;

            Ok(())
        }

        pub fn size() -> usize {
            5 + 8 // header + content
        }
    }

    /// Frame data descriptor (pixel data)
    pub struct DataDescriptor {
        pub data: Vec<u8>,
    }

    impl DataDescriptor {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            let header = BlockHeader {
                type_id: BlockType::FrameDataDescriptor as u8,
                length: self.data.len() as u32,
            };
            header.write(writer)?;
            writer.write_all(&self.data)?;
            Ok(())
        }
    }

    /// Frame offset descriptor (table of frame positions)
    pub struct FrameOffsetDescriptor {
        pub offsets: Vec<u32>,
    }

    impl FrameOffsetDescriptor {
        pub fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
            let header = BlockHeader {
                type_id: BlockType::FrameOffsetDescriptor as u8,
                length: (self.offsets.len() * 4) as u32,
            };
            header.write(writer)?;

            for offset in &self.offsets {
                write_u32_le(writer, *offset)?;
            }

            Ok(())
        }
    }
}

/// C code generation utilities for embedding QGF data
pub mod codegen {
    use super::*;

    /// Generate a formatted C byte array from raw data
    pub fn format_bytes(data: &[u8], newline_after: usize) -> String {
        let mut result = String::new();
        for (n, byte) in data.iter().enumerate() {
            if n % newline_after == 0 && n > 0 && n != data.len() {
                result.push_str("\n ");
            } else if n == 0 {
                result.push(' ');
            }
            result.push_str(&format!(" 0x{:02X},", byte));
        }
        result
    }

    /// Generate C header file content for QGF data
    pub fn generate_header(
        var_name: &str,
        byte_count: usize,
        license: &str,
    ) -> String {
        format!(
            "{license}#pragma once

#include <qp.h>

extern const uint32_t {var_name}_length;
extern const uint8_t {var_name}[{byte_count}];
"
        )
    }

    /// Generate C source file content for QGF data
    pub fn generate_source(
        var_name: &str,
        data: &[u8],
        metadata: &str,
        license: &str,
    ) -> String {
        let formatted_bytes = format_bytes(data, 16);
        format!(
            "{license}{metadata}#include <qp.h>

const uint32_t {var_name}_length = {};
// clang-format off
const uint8_t {var_name}[] = {{{formatted_bytes}
}};
// clang-format on
",
            data.len()
        )
    }

    /// Generate license header for generated code
    pub fn generate_license(year: u16, generated_type: &str, command: &str) -> String {
        format!(
            "// Copyright {} QMK -- generated source code only, {} retains original copyright
// SPDX-License-Identifier: GPL-2.0-or-later
// This file was auto-generated by `{}`

",
            year, generated_type, command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv() {
        let red = RgbColor::new(255, 0, 0);
        let (h, s, v) = red.to_qmk_hsv();
        assert_eq!(v, 255);
        assert_eq!(s, 255);
    }

    #[test]
    fn test_rgb_to_565() {
        let rgb = rgb_to565(255, 255, 255);
        assert_eq!(rgb, [0xFF, 0xFF]);
    }

    #[test]
    fn test_rle_compression() {
        let data = vec![1, 1, 1, 1, 2, 3, 4];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rescale_byte() {
        assert_eq!(rescale_byte(255, 31), 31);
        assert_eq!(rescale_byte(0, 31), 0);
        assert_eq!(rescale_byte(128, 255), 128);
    }
}
