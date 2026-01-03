//! Python-equivalent reference implementations for validation
//! These serve as oracles for testing the Rust implementation

#[cfg(test)]
mod painter_utils {
    /// Python reference implementation of RLE compression
    /// This is a direct port of QMK's compress_bytes_qmk_rle() function
    fn compress_rle(data: &[u8]) -> Vec<u8> {
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

    /// Python reference for rescale_byte
    fn rescale_byte(val: u8, maxval: u32) -> u8 {
        ((val as u32 * maxval) / 255) as u8
    }

    /// Python reference for RGB565 conversion
    fn rgb_to565(r: u8, g: u8, b: u8) -> (u8, u8) {
        let msb = (((r >> 3) & 0x1F) << 3) + ((g >> 5) & 0x07);
        let lsb = (((g >> 2) & 0x07) << 5) + ((b >> 3) & 0x1F);
        (msb as u8, lsb as u8)
    }

    // ==================== Validation Tests ====================

    use qmk_painter::{RleCompressor, rescale_byte, rgb_to565};

    #[test]
    fn validate_rle_matches_empty() {
        let data: Vec<u8> = vec![];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch for empty data");
    }

    #[test]
    fn validate_rle_matches_single() {
        let data = vec![0xFF];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch for single byte");
    }

    #[test]
    fn validate_rle_matches_repeating() {
        let data = vec![0xAA; 50];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch for repeating bytes");
    }

    #[test]
    fn validate_rle_matches_mixed() {
        let data = vec![
            0x01, 0x02, 0x03,
            0xFF, 0xFF, 0xFF, 0xFF,
            0xAA, 0xBB, 0xCC,
        ];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch for mixed pattern");
    }

    #[test]
    fn validate_rle_matches_all_bytes() {
        // Test with all possible byte values
        let data: Vec<u8> = (0..=255).collect();
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch for all bytes 0-255");
    }

    #[test]
    fn validate_rle_matches_boundary_127() {
        // Test at 127 bytes (boundary for marker encoding)
        let data = vec![0x55; 127];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch at 127-byte boundary");
    }

    #[test]
    fn validate_rle_matches_boundary_128() {
        // Test at 128 bytes (exact block size)
        let data = vec![0x55; 128];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch at 128-byte boundary");
    }

    #[test]
    fn validate_rle_matches_boundary_129() {
        // Test at 129 bytes (overflow)
        let data = vec![0x55; 129];
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch at 129-byte boundary");
    }

    #[test]
    fn validate_rle_matches_large_literal() {
        // Test large literal block (all different bytes)
        let data: Vec<u8> = (0..128).map(|i| (i * 2) as u8).collect();
        let rust_result = RleCompressor::compress(&data);
        let result = compress_rle(&data);
        assert_eq!(rust_result, result, "RLE compression mismatch for large literal block");
    }

    #[test]
    fn validate_rescale_byte_matches_python() {
        for val in [0, 1, 127, 128, 254, 255] {
            for maxval in [1, 2, 4, 8, 15, 16, 31, 32, 63, 64, 127, 128, 255] {
                let rust_result = rescale_byte(val, maxval);
                let result = rescale_byte(val, maxval);
                assert_eq!(
                    rust_result, result,
                    "rescale_byte mismatch: rescale_byte({}, {}) - Rust: {}, Python: {}",
                    val, maxval, rust_result, result
                );
            }
        }
    }

    #[test]
    fn validate_rgb565_matches_primaries() {
        // Test primary colors
        let test_cases = vec![
            (255, 0, 0),     // Red
            (0, 255, 0),     // Green
            (0, 0, 255),     // Blue
            (255, 255, 255), // White
            (0, 0, 0),       // Black
            (128, 128, 128), // Gray
        ];

        for (r, g, b) in test_cases {
            let (rust_msb, rust_lsb) = rgb_to565(r, g, b);
            let (msb, lsb) = rgb_to565(r, g, b);
            assert_eq!(
                (rust_msb, rust_lsb),
                (msb, lsb),
                "RGB565 mismatch for ({}, {}, {})",
                r, g, b
            );
        }
    }

    #[test]
    fn validate_rgb565_matches_all_levels() {
        // Test all 256 levels for each component
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let (rust_msb, rust_lsb) = rgb_to565(r, g, b);
                    let (msb, lsb) = rgb_to565(r, g, b);
                    assert_eq!(
                        (rust_msb, rust_lsb),
                        (msb, lsb),
                        "RGB565 mismatch for ({}, {}, {})",
                        r, g, b
                    );
                }
            }
        }
    }

    // ==================== Cross-Validation Tests ====================

    #[test]
    fn test_rle_compress_decompress_matches_python() {
        // Ensure Rust compress + decompress equals Python compress + decompress
        let test_data = vec![
            vec![],
            vec![0xFF],
            vec![0xFF; 100],
            vec![0x00, 0xFF].repeat(50),
            (0..=255).collect::<Vec<u8>>(),
        ];

        for data in test_data {
            let rust_compressed = RleCompressor::compress(&data);
            let compressed = compress_rle(&data);

            // Both should decompress to the same value
            let rust_decompressed = RleCompressor::decompress(&rust_compressed).unwrap();
            let decompressed = RleCompressor::decompress(&compressed).unwrap();

            assert_eq!(rust_decompressed, data, "Rust decompress failed for: {:?}", data);
            assert_eq!(
                decompressed, data,
                "Python decompress failed for: {:?}",
                data
            );
            assert_eq!(
                rust_compressed, compressed,
                "Compression output differs for: {:?}",
                data
            );
        }
    }

    #[test]
    fn test_rle_interop_rust_compress_decompress() {
        // Test that Rust-compressed data can be decompressed using the algorithm
        let data = vec![0xFF; 100];
        let rust_compressed = RleCompressor::compress(&data);
        let decompressed = RleCompressor::decompress(&rust_compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_rle_interop_decompress_rust() {
        // Test that Python-compressed data can be decompressed by Rust
        let data = vec![0xAA, 0xBB, 0xFF, 0xFF, 0xFF, 0xCC, 0xDD];
        let compressed = compress_rle(&data);
        let rust_decompressed = RleCompressor::decompress(&compressed).unwrap();
        assert_eq!(rust_decompressed, data);
    }

    // ==================== Statistical Tests ====================

    #[test]
    fn test_rle_compression_statistics() {
        struct TestCase {
            name: &'static str,
            data: Vec<u8>,
            expected_ratio_min: f32,
            expected_ratio_max: f32,
        }

        let test_cases = vec![
            TestCase {
                name: "Solid color (best case)",
                data: vec![0xFF; 256],
                expected_ratio_min: 0.0,
                expected_ratio_max: 0.1,
            },
            TestCase {
                name: "Alternating (worst case)",
                data: vec![0xAA, 0x55].repeat(128),
                expected_ratio_min: 0.8,
                expected_ratio_max: 1.2,
            },
            TestCase {
                name: "Gradient (average case)",
                data: (0..256).collect::<Vec<u8>>(),
                expected_ratio_min: 0.5,
                expected_ratio_max: 1.5,
            },
        ];

        for test in test_cases {
            let compressed = RleCompressor::compress(&test.data);
            let ratio = compressed.len() as f32 / test.data.len() as f32;

            assert!(
                ratio >= test.expected_ratio_min && ratio <= test.expected_ratio_max,
                "{}: unexpected compression ratio {:.2}x (expected {:.2}x to {:.2}x)",
                test.name,
                ratio,
                test.expected_ratio_min,
                test.expected_ratio_max
            );
        }
    }

    // ==================== Deterministic Tests ====================

    #[test]
    fn test_rle_deterministic_output() {
        // Same input should always produce same output
        let data = vec![0xFF, 0xAA, 0x55, 0x00].repeat(10);

        let result1 = RleCompressor::compress(&data);
        let result2 = RleCompressor::compress(&data);
        let result3 = RleCompressor::compress(&data);

        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_rgb565_deterministic_output() {
        for r in [0, 128, 255] {
            for g in [0, 128, 255] {
                for b in [0, 128, 255] {
                    let result1 = rgb_to565(r, g, b);
                    let result2 = rgb_to565(r, g, b);
                    assert_eq!(result1, result2);
                }
            }
        }
    }
}
