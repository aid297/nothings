use std::fs;
use std::path::Path;
use crate::compressions;
use crate::filesystem::file::File;

/// 创建临时测试文件
fn create_test_file(suffix: &str) -> String {
    let dir = format!("/tmp/aid_file_test_{}_{}", std::process::id(), suffix);
    let _ = fs::remove_file(&dir);
    dir
}

fn cleanup_file(path: &str) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_file_new_existing() {
    let path = create_test_file("new_existing");
    fs::write(&path, "hello world").unwrap();
    
    let file = File::new(&path);
    assert!(file.exist);
    assert!(file.is_file());
    assert_eq!(file.size, 11);
    assert_eq!(file.filename, path.rsplit('/').next().unwrap());
    assert!(file.err.is_none());
    
    cleanup_file(&path);
}

#[test]
fn test_file_new_non_existing() {
    let file = File::new("/tmp/aid_nonexistent_file_xyz");
    assert!(!file.exist);
    assert_eq!(file.size, 0);
}

#[test]
fn test_file_new_parses_ext() {
    let path = create_test_file("ext");
    let path_with_ext = format!("{}.txt", path);
    fs::write(&path_with_ext, "test").unwrap();
    
    let file = File::new(&path_with_ext);
    assert_eq!(file.ext, "txt");
    
    cleanup_file(&path_with_ext);
}

// ==================== File::write ====================

#[test]
fn test_file_write_new() {
    let path = create_test_file("write_new");
    let mut file = File::new(&path);
    assert!(!file.exist);
    
    let result = file.write(b"hello rust");
    assert!(result.is_ok());
    assert!(file.exist);
    assert_eq!(file.size, 10);
    
    // 验证文件内容
    let content = fs::read(&path).unwrap();
    assert_eq!(content, b"hello rust");
    
    cleanup_file(&path);
}

#[test]
fn test_file_write_overwrite() {
    let path = create_test_file("write_overwrite");
    fs::write(&path, "old content").unwrap();
    
    let mut file = File::new(&path);
    assert_eq!(file.size, 11);
    
    file.write(b"new").unwrap();
    assert_eq!(file.size, 3);
    
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "new");
    
    cleanup_file(&path);
}

// ==================== File::read ====================

#[test]
fn test_file_read() {
    let path = create_test_file("read");
    fs::write(&path, "test data").unwrap();
    
    let file = File::new(&path);
    let result = file.read();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), b"test data");
    
    cleanup_file(&path);
}

#[test]
fn test_file_read_non_existing() {
    let file = File::new("/tmp/aid_nonexistent_read_xyz");
    let result = file.read();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "文件不存在");
}

// ==================== File::remove ====================

#[test]
fn test_file_remove() {
    let path = create_test_file("remove");
    fs::write(&path, "to delete").unwrap();
    
    let file = File::new(&path);
    assert!(file.exist);
    
    let result = file.remove();
    assert!(result.is_ok());
    assert!(!Path::new(&path).exists());
}

#[test]
fn test_file_remove_non_existing() {
    let file = File::new("/tmp/aid_nonexistent_remove_xyz");
    let result = file.remove();
    assert!(result.is_err());
}

// ==================== File::mv ====================

#[test]
fn test_file_mv() {
    let src = create_test_file("mv_src");
    let dst = create_test_file("mv_dst");
    fs::write(&src, "move me").unwrap();
    
    let mut file = File::new(&src);
    file.mv(&dst);
    assert!(file.err.is_none());
    assert_eq!(file.full_path, dst);
    assert!(!Path::new(&src).exists());
    assert!(Path::new(&dst).exists());
    
    // 验证内容未变
    let content = fs::read_to_string(&dst).unwrap();
    assert_eq!(content, "move me");
    
    cleanup_file(&dst);
}

#[test]
fn test_file_mv_target_exists() {
    let src = create_test_file("mv_src2");
    let dst = create_test_file("mv_dst2");
    fs::write(&src, "data").unwrap();
    fs::write(&dst, "existing").unwrap();
    
    let mut file = File::new(&src);
    file.mv(&dst);
    assert!(file.err.is_some());
    assert!(file.err.unwrap().contains("目标文件已存在"));
    
    cleanup_file(&src);
    cleanup_file(&dst);
}

// ==================== File::zip / File::unzip ====================

#[test]
fn test_file_zip_with_lz4() {
    let src = create_test_file("zip_lz4");
    let zip_file = create_test_file("zip_lz4_out");
    fs::write(&src, "compress me with lz4").unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_lz4();
    let result = file.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());
    
    let compressed = result.unwrap();
    assert!(compressed.exist);
    assert!(compressed.size > 0);
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
}

#[test]
fn test_file_zip_with_zlib() {
    let src = create_test_file("zip_zlib");
    let zip_file = format!("{}.zlib", create_test_file("zip_zlib_out"));
    fs::write(&src, "compress me with zlib").unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_zlib();
    let result = file.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
}

#[test]
fn test_file_zip_with_zstd() {
    let src = create_test_file("zip_zstd");
    let zip_file = format!("{}.zstd", create_test_file("zip_zstd_out"));
    fs::write(&src, "compress me with zstd").unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_zstd();
    let result = file.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
}

#[test]
fn test_file_zip_with_level() {
    let src = create_test_file("zip_level");
    let zip_file = format!("{}.zstd", create_test_file("zip_level_out"));
    fs::write(&src, "compress with level").unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_zstd_level(19);
    let result = file.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
}

#[test]
fn test_file_zip_non_existing() {
    let file = File::new("/tmp/aid_nonexistent_zip_xyz");
    let compressor = compressions::with_lz4();
    let result = file.zip("/tmp/out.lz4", compressor.as_ref());
    assert!(result.is_err());
}

#[test]
fn test_file_unzip_with_lz4() {
    let src = create_test_file("unzip_src");
    let zip_file = create_test_file("unzip_archive");
    let output = create_test_file("unzip_output");
    let original_content = "hello world unzip test";
    fs::write(&src, original_content).unwrap();
    
    // 压缩
    let file = File::new(&src);
    let compressor = compressions::with_lz4();
    file.zip(&zip_file, compressor.as_ref()).unwrap();
    
    // 解压
    let compressor = compressions::with_lz4();
    let result = File::unzip(&zip_file, &output, compressor.as_ref());
    assert!(result.is_ok());
    
    let restored = result.unwrap();
    assert!(restored.exist);
    let content = fs::read_to_string(&output).unwrap();
    assert_eq!(content, original_content);
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
    cleanup_file(&output);
}

#[test]
fn test_file_unzip_with_zlib() {
    let src = create_test_file("unzip_zlib_src");
    let zip_file = format!("{}.zlib", create_test_file("unzip_zlib_archive"));
    let output = create_test_file("unzip_zlib_output");
    let original = "zlib test content";
    fs::write(&src, original).unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_zlib();
    file.zip(&zip_file, compressor.as_ref()).unwrap();
    
    let compressor = compressions::with_zlib();
    let result = File::unzip(&zip_file, &output, compressor.as_ref());
    assert!(result.is_ok());
    
    let content = fs::read_to_string(&output).unwrap();
    assert_eq!(content, original);
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
    cleanup_file(&output);
}

#[test]
fn test_file_unzip_with_zstd() {
    let src = create_test_file("unzip_zstd_src");
    let zip_file = format!("{}.zstd", create_test_file("unzip_zstd_archive"));
    let output = create_test_file("unzip_zstd_output");
    let original = "zstd test content 中文测试";
    fs::write(&src, original).unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_zstd();
    file.zip(&zip_file, compressor.as_ref()).unwrap();
    
    let compressor = compressions::with_zstd();
    let result = File::unzip(&zip_file, &output, compressor.as_ref());
    assert!(result.is_ok());
    
    let content = fs::read_to_string(&output).unwrap();
    assert_eq!(content, original);
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
    cleanup_file(&output);
}

#[test]
fn test_file_unzip_non_existing() {
    let compressor = compressions::with_lz4();
    let result = File::unzip("/tmp/aid_nonexistent.lz4", "/tmp/aid_output", compressor.as_ref());
    assert!(result.is_err());
    match result {
        Err(e) => assert!(e.contains("读取压缩文件失败")),
        Ok(_) => panic!("期望返回错误"),
    }
}

#[test]
fn test_file_zip_unzip_roundtrip() {
    let src = create_test_file("roundtrip_src");
    let zip_file = create_test_file("roundtrip_zip");
    let output = create_test_file("roundtrip_output");
    
    // 写入较大内容测试完整性
    let original: String = (0..1000).map(|i| format!("line {}: test data\n", i)).collect();
    fs::write(&src, &original).unwrap();
    
    let file = File::new(&src);
    let compressor = compressions::with_zstd_level(9);
    file.zip(&zip_file, compressor.as_ref()).unwrap();
    
    let compressor = compressions::with_zstd();
    File::unzip(&zip_file, &output, compressor.as_ref()).unwrap();
    
    let restored = fs::read_to_string(&output).unwrap();
    assert_eq!(restored, original);
    
    cleanup_file(&src);
    cleanup_file(&zip_file);
    cleanup_file(&output);
}
