#[cfg(test)]
mod parity_tests {
    use qmk_painter::{
        ImageFormat, RleCompressor, RgbColor, convert_image_bytes, rescale_byte, rgb_to565,
        qgf, codegen,
    };
    use std::io::Cursor;

    use crate::painter::qmk_painter;

    // ==================== RLE Compression Tests ====================

    #[test]
    fn test_rle_empty_input() {
        let data: Vec<u8> = vec![];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_single_byte() {
        let data = vec![0xFF];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_two_identical_bytes() {
        let data = vec![0xAA, 0xAA];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_all_identical_bytes() {
        // 100 identical bytes should compress well
        let data = vec![0xFF; 100];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
        // Should be much smaller than original
        assert!(compressed.len() < data.len() / 5);
    }

    #[test]
    fn test_rle_all_different_bytes() {
        // 128 different bytes (fits in non-repeat block)
        let data: Vec<u8> = (0..128).collect();
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_mixed_repeat_and_literal() {
        // Pattern: AAAA BB CCCC DD EE
        let data = vec![
            0xAA, 0xAA, 0xAA, 0xAA,
            0xBB, 0xBB,
            0xCC, 0xCC, 0xCC, 0xCC,
            0xDD, 0xDD,
            0xEE, 0xEE,
        ];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_long_repeat_block() {
        // 128 identical bytes (maximum non-repeat block)
        let data = vec![0x55; 128];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_over_128_bytes_same() {
        // More than 128 identical bytes (must split into multiple blocks)
        let data = vec![0x77; 200];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_alternating_pattern() {
        // ABABABAB... pattern (worst case for RLE)
        let data = vec![0xAA, 0xBB].repeat(50);
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_grayscale_image() {
        // Simulate a simple grayscale gradient
        let data: Vec<u8> = (0..=255).cycle().take(256).collect();
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_qmk_example_from_docs() {
        // Example from QMK documentation
        // 0x04 0xFF means: repeat 0xFF 4 times
        // 0x83 0x12 0x34 0x56 means: literal 3 bytes (0x12, 0x34, 0x56)
        let expected = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x12, 0x34, 0x56];
        let compressed = vec![0x04, 0xFF, 0x83, 0x12, 0x34, 0x56];
        
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, expected);
    }

    #[test]
    fn test_rle_compression_increases_for_random() {
        // Random-looking data usually doesn't compress
        let data = vec![
            0xFF, 0x00, 0xAA, 0x55, 0x12, 0x34, 0x56, 0x78,
            0xFF, 0x00, 0xAA, 0x55, 0x12, 0x34, 0x56, 0x78,
        ];
        let compressed = RleCompressor::compress(&data);
        // Compressed might be equal or slightly larger due to overhead
        assert!(compressed.len() <= data.len() + 10);
    }

    // ==================== Color Conversion Tests ====================

    #[test]
    fn test_rescale_byte_zero() {
        assert_eq!(rescale_byte(0, 255), 0);
        assert_eq!(rescale_byte(0, 31), 0);
        assert_eq!(rescale_byte(0, 15), 0);
    }

    #[test]
    fn test_rescale_byte_max() {
        assert_eq!(rescale_byte(255, 255), 255);
        assert_eq!(rescale_byte(255, 31), 31);
        assert_eq!(rescale_byte(255, 15), 15);
    }

    #[test]
    fn test_rescale_byte_mid() {
        // 128 (50%) should scale proportionally
        assert_eq!(rescale_byte(128, 31), 15); // 128/255*31 ≈ 15.6
        assert_eq!(rescale_byte(128, 15), 7); // 128/255*15 ≈ 7.5
    }

    #[test]
    fn test_rgb_to_565_white() {
        let [msb, lsb] = rgb_to565(255, 255, 255);
        // All bits set in RGB565
        assert_eq!(msb, 0xFF);
        assert_eq!(lsb, 0xFF);
    }

    #[test]
    fn test_rgb_to_565_black() {
        let [msb, lsb] = rgb_to565(0, 0, 0);
        assert_eq!(msb, 0x00);
        assert_eq!(lsb, 0x00);
    }

    #[test]
    fn test_rgb_to_565_red() {
        let [msb, lsb] = rgb_to565(255, 0, 0);
        // Red should have R bits set, G and B zero
        let value = u16::from_le_bytes([lsb, msb]);
        let r = (value & 0xF800) >> 11;
        let g = (value & 0x07E0) >> 5;
        let b = value & 0x001F;
        assert_eq!(r, 31);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_rgb_to_565_green() {
        let [msb, lsb] = rgb_to565(0, 255, 0);
        let value = u16::from_le_bytes([lsb, msb]);
        let r = (value & 0xF800) >> 11;
        let g = (value & 0x07E0) >> 5;
        let b = value & 0x001F;
        assert_eq!(r, 0);
        assert_eq!(g, 63);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_rgb_to_565_blue() {
        let [msb, lsb] = rgb_to565(0, 0, 255);
        let value = u16::from_le_bytes([lsb, msb]);
        let r = (value & 0xF800) >> 11;
        let g = (value & 0x07E0) >> 5;
        let b = value & 0x001F;
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 31);
    }

    #[test]
    fn test_rgb_color_new() {
        let color = RgbColor::new(128, 64, 32);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let red = RgbColor::new(255, 0, 0);
        let (h, s, v) = red.to_qmk_hsv();
        assert_eq!(v, 255); // Max value
        assert_eq!(s, 255); // Full saturation
        // Hue should be near 0 (red) - actual value depends on hue calculation
    }

    #[test]
    fn test_rgb_to_hsv_white() {
        let white = RgbColor::new(255, 255, 255);
        let (_h, s, v) = white.to_qmk_hsv();
        assert_eq!(v, 255);
        assert_eq!(s, 0); // No saturation (neutral)
    }

    #[test]
    fn test_rgb_to_hsv_black() {
        let black = RgbColor::new(0, 0, 0);
        let (_h, s, v) = black.to_qmk_hsv();
        assert_eq!(v, 0);
    }

    #[test]
    fn test_rgb_to_hsv_gray() {
        let gray = RgbColor::new(128, 128, 128);
        let (_h, s, v) = gray.to_qmk_hsv();
        assert_eq!(s, 0); // Neutral gray has no saturation
        assert_eq!(v, 128); // Mid-value
    }

    // ==================== Image Format Tests ====================

    #[test]
    fn test_format_rgb888_metadata() {
        let format = ImageFormat::Rgb888.metadata();
        assert_eq!(format.bpp, 24);
        assert_eq!(format.num_colors, 16777216);
        assert!(!format.has_palette);
        assert_eq!(format.image_format_byte, 0x09);
    }

    #[test]
    fn test_format_rgb565_metadata() {
        let format = ImageFormat::Rgb565.metadata();
        assert_eq!(format.bpp, 16);
        assert_eq!(format.num_colors, 65536);
        assert!(!format.has_palette);
        assert_eq!(format.image_format_byte, 0x08);
    }

    #[test]
    fn test_format_pal256_metadata() {
        let format = ImageFormat::Pal256.metadata();
        assert_eq!(format.bpp, 8);
        assert_eq!(format.num_colors, 256);
        assert!(format.has_palette);
        assert_eq!(format.image_format_byte, 0x07);
    }

    #[test]
    fn test_format_pal16_metadata() {
        let format = ImageFormat::Pal16.metadata();
        assert_eq!(format.bpp, 4);
        assert_eq!(format.num_colors, 16);
        assert!(format.has_palette);
        assert_eq!(format.image_format_byte, 0x06);
    }

    #[test]
    fn test_format_mono256_metadata() {
        let format = ImageFormat::Mono256.metadata();
        assert_eq!(format.bpp, 8);
        assert_eq!(format.num_colors, 256);
        assert!(!format.has_palette);
        assert_eq!(format.image_format_byte, 0x03);
    }

    #[test]
    fn test_format_mono16_metadata() {
        let format = ImageFormat::Mono16.metadata();
        assert_eq!(format.bpp, 4);
        assert_eq!(format.num_colors, 16);
        assert!(!format.has_palette);
        assert_eq!(format.image_format_byte, 0x02);
    }

    #[test]
    fn test_format_mono4_metadata() {
        let format = ImageFormat::Mono4.metadata();
        assert_eq!(format.bpp, 2);
        assert_eq!(format.num_colors, 4);
        assert!(!format.has_palette);
        assert_eq!(format.image_format_byte, 0x01);
    }

    #[test]
    fn test_format_mono2_metadata() {
        let format = ImageFormat::Mono2.metadata();
        assert_eq!(format.bpp, 1);
        assert_eq!(format.num_colors, 2);
        assert!(!format.has_palette);
        assert_eq!(format.image_format_byte, 0x00);
    }

    // ==================== Image Conversion Tests ====================

    #[test]
    fn test_convert_mono2_single_pixel() {
        let pixels = vec![255]; // White
        let format = ImageFormat::Mono2.metadata();
        let converted = convert_image_bytes(&pixels, 1, 1, &format).unwrap();
        // 1 pixel in 1-bit format = 1 byte with upper 7 bits zero
        assert_eq!(converted.pixel_data.len(), 1);
        assert_eq!(converted.pixel_data[0], 0x01); // Binary 00000001
    }

    #[test]
    fn test_convert_mono4_4_pixels() {
        let pixels = vec![0xFF, 0x80, 0x40, 0x00];
        let format = ImageFormat::Mono4.metadata();
        let converted = convert_image_bytes(&pixels, 4, 1, &format).unwrap();
        // 4 pixels in 2-bit format = 2 bytes
        assert_eq!(converted.pixel_data.len(), 2);
    }

    #[test]
    fn test_convert_mono16_2_pixels() {
        let pixels = vec![0xFF, 0x00];
        let format = ImageFormat::Mono16.metadata();
        let converted = convert_image_bytes(&pixels, 2, 1, &format).unwrap();
        // 2 pixels in 4-bit format = 1 byte
        assert_eq!(converted.pixel_data.len(), 1);
    }

    #[test]
    fn test_convert_mono256_8_pixels() {
        let pixels = vec![0xFF, 0xCC, 0x99, 0x66, 0x33, 0x00, 0x12, 0x34];
        let format = ImageFormat::Mono256.metadata();
        let converted = convert_image_bytes(&pixels, 8, 1, &format).unwrap();
        // 8 pixels in 8-bit format = 8 bytes
        assert_eq!(converted.pixel_data.len(), 8);
        assert_eq!(converted.pixel_data, pixels);
    }

    #[test]
    fn test_convert_rgb565_single_pixel() {
        // Red pixel in RGB format (3 bytes input)
        let pixels = vec![255, 0, 0];
        let format = ImageFormat::Rgb565.metadata();
        let converted = convert_image_bytes(&pixels, 1, 1, &format).unwrap();
        // RGB565 = 2 bytes
        assert_eq!(converted.pixel_data.len(), 2);
    }

    #[test]
    fn test_convert_rgb565_multiple_colors() {
        // Red, Green, Blue
        let pixels = vec![
            255, 0, 0,     // Red
            0, 255, 0,     // Green
            0, 0, 255,     // Blue
        ];
        let format = ImageFormat::Rgb565.metadata();
        let converted = convert_image_bytes(&pixels, 3, 1, &format).unwrap();
        assert_eq!(converted.pixel_data.len(), 6); // 3 pixels × 2 bytes
    }

    #[test]
    fn test_convert_rgb888_direct() {
        let pixels = vec![255, 0, 0];
        let format = ImageFormat::Rgb888.metadata();
        let converted = convert_image_bytes(&pixels, 1, 1, &format).unwrap();
        assert_eq!(converted.pixel_data, pixels);
    }

    #[test]
    fn test_convert_palette_indices() {
        // Palette indices directly used
        let indices = vec![0, 1, 2, 3];
        let format = ImageFormat::Pal256.metadata();
        let converted = convert_image_bytes(&indices, 4, 1, &format).unwrap();
        assert_eq!(converted.pixel_data, indices);
    }

    // ==================== QGF Block Header Tests ====================

    #[test]
    fn test_qgf_block_header_write() {
        let header = qgf::BlockHeader {
            type_id: 0x00,
            length: 10,
        };
        
        let mut buffer = Cursor::new(Vec::new());
        header.write(&mut buffer).unwrap();
        
        let bytes = buffer.into_inner();
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], 0x00); // type_id
        assert_eq!(bytes[1], 0xFF); // ~type_id
        // Bytes 2-4 are length in little-endian
        assert_eq!(bytes[2], 0x0A); // 10 in little-endian (0x0A)
    }

    #[test]
    fn test_qgf_graphics_descriptor_size() {
        assert_eq!(qgf::GraphicsDescriptor::size(), 23); // 5 header + 18 content
    }

    #[test]
    fn test_qgf_frame_descriptor_size() {
        assert_eq!(qgf::FrameDescriptor::size(), 11); // 5 header + 6 content
    }

    #[test]
    fn test_qgf_delta_descriptor_size() {
        assert_eq!(qgf::DeltaDescriptor::size(), 13); // 5 header + 8 content
    }

    #[test]
    fn test_qgf_graphics_descriptor_write() {
        let descriptor = qgf::GraphicsDescriptor {
            version: 1,
            total_file_size: 100,
            image_width: 32,
            image_height: 32,
            frame_count: 1,
        };
        
        let mut buffer = Cursor::new(Vec::new());
        descriptor.write(&mut buffer).unwrap();
        
        let bytes = buffer.into_inner();
        assert_eq!(bytes.len(), 23);
        
        // Check magic number (0x464751 = "QGF" in little-endian)
        assert_eq!(bytes[5], 0x51); // 'Q'
        assert_eq!(bytes[6], 0x47); // 'G'
        assert_eq!(bytes[7], 0x46); // 'F'
    }

    #[test]
    fn test_qgf_frame_descriptor_transparency_flag() {
        let mut descriptor = qgf::FrameDescriptor {
            format: 0x08,
            flags: 0,
            compression: 0x01,
            transparency_index: 0xFF,
            delay: 1000,
        };
        
        assert!(!descriptor.is_transparent());
        descriptor.set_transparent(true);
        assert!(descriptor.is_transparent());
        assert_eq!(descriptor.flags & 0x01, 0x01);
    }

    #[test]
    fn test_qgf_frame_descriptor_delta_flag() {
        let mut descriptor = qgf::FrameDescriptor {
            format: 0x08,
            flags: 0,
            compression: 0x00,
            transparency_index: 0xFF,
            delay: 0,
        };
        
        assert!(!descriptor.is_delta());
        descriptor.set_delta(true);
        assert!(descriptor.is_delta());
        assert_eq!(descriptor.flags & 0x02, 0x02);
    }

    // ==================== Code Generation Tests ====================

    #[test]
    fn test_codegen_format_bytes() {
        let data = vec![0xFF, 0x00, 0xAA, 0x55];
        let formatted = codegen::format_bytes(&data, 2);
        
        // Should contain the bytes in hex format
        assert!(formatted.contains("0xFF"));
        assert!(formatted.contains("0x00"));
        assert!(formatted.contains("0xAA"));
        assert!(formatted.contains("0x55"));
    }

    #[test]
    fn test_codegen_generate_header() {
        let license = "// Test license\n";
        let header = codegen::generate_header("test_image", 1024, license);
        
        assert!(header.contains("test_image"));
        assert!(header.contains("1024"));
        assert!(header.contains("#pragma once"));
        assert!(header.contains("#include <qp.h>"));
        assert!(header.contains("extern"));
    }

    #[test]
    fn test_codegen_generate_source() {
        let license = "// Test license\n";
        let data = vec![0xFF, 0x00, 0xAA];
        let source = codegen::generate_source("test_image", &data, "// Metadata", license);
        
        assert!(source.contains("test_image"));
        assert!(source.contains("0xFF"));
        assert!(source.contains("const uint8_t"));
    }

    #[test]
    fn test_codegen_license_header() {
        let license = codegen::generate_license(2025, "image", "test-command");
        
        assert!(license.contains("2025"));
        assert!(license.contains("image"));
        assert!(license.contains("test-command"));
        assert!(license.contains("GPL-2.0-or-later"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_workflow_small_image() {
        // 2x2 grayscale image
        let pixels = vec![255, 128, 64, 0];
        let format = ImageFormat::Mono16.metadata();
        
        // Convert
        let converted = convert_image_bytes(&pixels, 2, 2, &format).unwrap();
        assert_eq!(converted.pixel_data.len(), 2); // 4 pixels in 4-bit = 2 bytes
        
        // Compress
        let compressed = RleCompressor::compress(&converted.pixel_data);
        
        // Decompress and verify round-trip
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, converted.pixel_data);
    }

    #[test]
    fn test_full_workflow_with_qgf() {
        let mut qgf_file = Cursor::new(Vec::new());
        
        // Graphics descriptor
        let graphics = qgf::GraphicsDescriptor {
            version: 1,
            total_file_size: 0,
            image_width: 16,
            image_height: 16,
            frame_count: 1,
        };
        graphics.write(&mut qgf_file).unwrap();
        
        // Frame offsets
        let offsets = qgf::FrameOffsetDescriptor {
            offsets: vec![23],
        };
        offsets.write(&mut qgf_file).unwrap();
        
        // Frame descriptor
        let frame = qgf::FrameDescriptor {
            format: 0x08,
            flags: 0,
            compression: 0x01,
            transparency_index: 0xFF,
            delay: 1000,
        };
        frame.write(&mut qgf_file).unwrap();
        
        // Verify file was written
        let bytes = qgf_file.into_inner();
        assert!(bytes.len() > 0);
        assert!(bytes.len() >= 23 + 5 + 4 + 11); // graphics + offsets + frame
    }

    #[test]
    fn test_rle_compression_efficiency() {
        // Test that solid color images compress well
        let solid = vec![0xFF; 1024];
        let compressed = RleCompressor::compress(&solid);
        
        // Should compress to approximately 2-3 bytes
        assert!(compressed.len() < 10);
    }

    #[test]
    fn test_rle_no_false_decompression_errors() {
        // Test various patterns don't cause decompression errors
        let patterns = vec![
            vec![0x00; 256],
            vec![0xFF; 256],
            (0..=255).collect::<Vec<u8>>(),
            (0..=255).rev().collect::<Vec<u8>>(),
            (0..128).chain(0..128).collect::<Vec<u8>>(),
        ];
        
        for pattern in patterns {
            let compressed = RleCompressor::compress(&pattern);
            let decompressed = RleCompressor::decompress(&compressed);
            assert!(decompressed.is_ok());
            assert_eq!(decompressed.unwrap(), pattern);
        }
    }
}

// ==================== Regression Tests ====================
// These test specific bugs or edge cases found during development

#[cfg(test)]
mod regression_tests {
    use qmk_painter::RleCompressor;

    use crate::painter::qmk_painter;

    #[test]
    fn test_rle_issue_buffer_boundary() {
        // Regression: ensure 128-byte boundary is handled correctly
        let data = vec![0xAA; 128];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_issue_repeat_after_literal() {
        // Regression: repeat block immediately after literal block
        let data = vec![0x01, 0x02, 0x03, 0xFF, 0xFF, 0xFF, 0xFF];
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_issue_exact_128_literal() {
        // Regression: exactly 128 literal bytes
        let data: Vec<u8> = (0..128).collect();
        let compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
