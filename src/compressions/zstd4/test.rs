use crate::compressions::zstd4::app::Zstd;

// ─────────────────────────────────────────────
// compress / decompress
// ─────────────────────────────────────────────

#[test]
fn test_compress_decompress() {
    let data = b"hello world, this is a test for zstd compression";
    let compressed = Zstd::compress(data).unwrap();
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_compress_empty() {
    let data = b"";
    let compressed = Zstd::compress(data).unwrap();
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_compress_large_data() {
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let compressed = Zstd::compress(&data).unwrap();
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_compress_repetitive_data() {
    let data = "abcdefghij".repeat(1000);
    let compressed = Zstd::compress(data.as_bytes()).unwrap();
    // 重复数据应该有较好的压缩率
    assert!(compressed.len() < data.len());
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data.as_bytes());
}

#[test]
fn test_decompress_invalid_data() {
    let invalid = b"this is not zstd compressed data";
    assert!(Zstd::decompress(invalid).is_err());
}

// ─────────────────────────────────────────────
// compress_with_level
// ─────────────────────────────────────────────

#[test]
fn test_compress_with_level_1() {
    let data = b"hello world, level 1 fastest compression test";
    let compressed = Zstd::compress_with_level(data, 1).unwrap();
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_compress_with_level_10() {
    let data = b"hello world, level 10 balanced compression test";
    let compressed = Zstd::compress_with_level(data, 10).unwrap();
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_compress_with_level_21() {
    let data = b"hello world, level 21 max compression test";
    let compressed = Zstd::compress_with_level(data, 21).unwrap();
    let decompressed = Zstd::decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_higher_level_better_compression() {
    let data = "the quick brown fox jumps over the lazy dog".repeat(500);
    let compressed_l1 = Zstd::compress_with_level(data.as_bytes(), 1).unwrap();
    let compressed_l21 = Zstd::compress_with_level(data.as_bytes(), 21).unwrap();
    // 高级别压缩结果应该 <= 低级别
    assert!(compressed_l21.len() <= compressed_l1.len());
}

// ─────────────────────────────────────────────
// compress_file / decompress_file
// ─────────────────────────────────────────────

#[test]
fn test_compress_decompress_file() {
    let input_path = "/tmp/zstd_test_input.txt";
    let compressed_path = "/tmp/zstd_test_compressed.zst";
    let output_path = "/tmp/zstd_test_output.txt";

    let original = "hello world, file compression test!";
    std::fs::write(input_path, original).unwrap();

    Zstd::compress_file(input_path, compressed_path).unwrap();
    Zstd::decompress_file(compressed_path, output_path).unwrap();

    let result = std::fs::read_to_string(output_path).unwrap();
    assert_eq!(result, original);

    // 清理
    let _ = std::fs::remove_file(input_path);
    let _ = std::fs::remove_file(compressed_path);
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn test_compress_file_with_level() {
    let input_path = "/tmp/zstd_test_level_input.txt";
    let compressed_path = "/tmp/zstd_test_level_compressed.zst";
    let output_path = "/tmp/zstd_test_level_output.txt";

    let original = "hello world, file compression with level 15!";
    std::fs::write(input_path, original).unwrap();

    Zstd::compress_file_with_level(input_path, compressed_path, 15).unwrap();
    Zstd::decompress_file(compressed_path, output_path).unwrap();

    let result = std::fs::read_to_string(output_path).unwrap();
    assert_eq!(result, original);

    // 清理
    let _ = std::fs::remove_file(input_path);
    let _ = std::fs::remove_file(compressed_path);
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn test_compress_file_not_found() {
    let result = Zstd::compress_file("/tmp/nonexistent_zstd_12345.txt", "/tmp/out.zst");
    assert!(result.is_err());
}

#[test]
fn test_decompress_file_not_found() {
    let result = Zstd::decompress_file("/tmp/nonexistent_zstd_12345.zst", "/tmp/out.txt");
    assert!(result.is_err());
}

// ─────────────────────────────────────────────
// compression_ratio
// ─────────────────────────────────────────────

#[test]
fn test_compression_ratio_empty() {
    let ratio = Zstd::compression_ratio(&[], &[1, 2, 3]);
    assert_eq!(ratio, 0.0);
}

#[test]
fn test_compression_ratio_normal() {
    let original = vec![0u8; 1000];
    let compressed = vec![0u8; 100];
    let ratio = Zstd::compression_ratio(&original, &compressed);
    assert!((ratio - 0.1).abs() < f64::EPSILON);
}

#[test]
fn test_compression_ratio_with_real_data() {
    let data = "abcdefghij".repeat(1000);
    let compressed = Zstd::compress(data.as_bytes()).unwrap();
    let ratio = Zstd::compression_ratio(data.as_bytes(), &compressed);
    // 重复数据压缩率应该 < 1.0
    assert!(ratio < 1.0);
}
