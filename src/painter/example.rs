//! Example usage of the QMK Painter module
//!
//! This example demonstrates:
//! 1. Image format conversion
//! 2. Pixel data encoding
//! 3. RLE compression
//! 4. QGF file generation
//! 5. C code generation for embedding

use qmk_painter::{
    ImageFormat, RleCompressor, RgbColor, codegen, qgf,
    convert_image_bytes, FormatMetadata,
};
use std::io::Cursor;

fn main() -> std::io::Result<()> {
    // Example 1: RGB to HSV conversion
    println!("=== Color Space Conversion ===");
    let red = RgbColor::new(255, 0, 0);
    let (h, s, v) = red.to_qmk_hsv();
    println!("Red (255,0,0) -> HSV({},{},{})", h, s, v);

    let white = RgbColor::new(255, 255, 255);
    let (h, s, v) = white.to_qmk_hsv();
    println!("White (255,255,255) -> HSV({},{},{})", h, s, v);

    // Example 2: RLE Compression
    println!("\n=== RLE Compression ===");
    let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xAA, 0xBB, 0xCC];
    println!("Original data: {:02X?}", data);
    let compressed = RleCompressor::compress(&data);
    println!("Compressed: {:02X?}", compressed);
    let decompressed = RleCompressor::decompress(&compressed).unwrap();
    println!("Decompressed: {:02X?}", decompressed);
    println!(
        "Compression ratio: {:.1}%",
        (compressed.len() as f32 / data.len() as f32) * 100.0
    );

    // Example 3: Creating a simple QGF file
    println!("\n=== QGF File Generation ===");
    let mut qgf_file = Cursor::new(Vec::new());

    // Create graphics descriptor
    let graphics_desc = qgf::GraphicsDescriptor {
        version: 1,
        total_file_size: 0, // Will be filled in later
        image_width: 32,
        image_height: 32,
        frame_count: 1,
    };
    graphics_desc.write(&mut qgf_file)?;
    println!(
        "Graphics descriptor written ({} bytes)",
        qgf::GraphicsDescriptor::size()
    );

    // Create frame offset descriptor
    let frame_offsets = qgf::FrameOffsetDescriptor {
        offsets: vec![qgf::GraphicsDescriptor::size() as u32 + 13], // Placeholder offset
    };
    frame_offsets.write(&mut qgf_file)?;
    println!(
        "Frame offsets written ({} bytes)",
        frame_offsets.offsets.len() * 4 + 5
    );

    // Create frame descriptor
    let mut frame_desc = qgf::FrameDescriptor {
        format: ImageFormat::Rgb565.metadata().image_format_byte,
        flags: 0,
        compression: 0x01, // RLE enabled
        transparency_index: 0xFF,
        delay: 1000,
    };
    frame_desc.write(&mut qgf_file)?;
    println!(
        "Frame descriptor written ({} bytes)",
        qgf::FrameDescriptor::size()
    );

    // Example data (32x32 RGB565 = 2048 bytes)
    let test_data = vec![0xFF; 2048];
    let compressed_data = RleCompressor::compress(&test_data);
    let data_desc = qgf::DataDescriptor {
        data: compressed_data.clone(),
    };
    data_desc.write(&mut qgf_file)?;
    println!(
        "Frame data written ({} bytes, compressed from {})",
        compressed_data.len(),
        test_data.len()
    );

    let total_size = qgf_file.position() as u32;
    println!("Total QGF file size: {} bytes", total_size);

    // Example 4: C code generation
    println!("\n=== C Code Generation ===");
    let license = codegen::generate_license(2025, "image", "qmk painter-convert-graphics");
    let header = codegen::generate_header("gfx_my_image", 100, &license);
    println!("Generated header file preview:");
    println!("{}", header);

    // Generate byte array formatting
    let sample_bytes = vec![0xFF, 0x00, 0xAA, 0x55];
    let formatted = codegen::format_bytes(&sample_bytes, 8);
    println!("Formatted byte array: {}", formatted);

    Ok(())
}

/// Advanced example: Converting and compressing a conceptual image
#[allow(dead_code)]
fn example_image_conversion() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a simple 2x2 grayscale image (4 pixels: 0xFF, 0x80, 0x40, 0x00)
    let pixel_data = vec![0xFF, 0x80, 0x40, 0x00];
    let width = 2u32;
    let height = 2u32;

    // Convert to mono16 (4-bit grayscale)
    let format = ImageFormat::Mono16.metadata();
    let converted = convert_image_bytes(&pixel_data, width, height, &format)?;

    println!("Original pixels: {:?}", pixel_data);
    println!("Converted to {}: {:02X?}", format.image_format, converted.pixel_data);

    // Apply RLE compression
    let compressed = RleCompressor::compress(&converted.pixel_data);
    println!("RLE compressed: {:02X?}", compressed);
    println!(
        "Original size: {}, Compressed size: {}, Ratio: {:.1}%",
        converted.pixel_data.len(),
        compressed.len(),
        (compressed.len() as f32 / converted.pixel_data.len() as f32) * 100.0
    );

    Ok(())
}
