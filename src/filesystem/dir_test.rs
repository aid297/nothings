use crate::filesystem::dir::Dir;
use crate::compressions;
use std::fs;
use std::path::Path;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 创建临时测试目录（带唯一后缀避免冲突）
fn create_test_dir(suffix: &str) -> String {
    let dir = format!("/tmp/aid_dir_test_{}_{}", std::process::id(), suffix);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// 清理测试目录
fn cleanup(path: &str) {
    let _ = fs::remove_dir_all(path);
}

/// 创建包含文件和子目录的测试目录结构
/// 结构:
///   base/
///     file1.txt ("hello")
///     file2.txt ("world")
///     sub1/
///       file3.txt ("nested")
fn setup_test_tree(base: &str) {
    fs::create_dir_all(format!("{}/sub1", base)).unwrap();
    fs::write(format!("{}/file1.txt", base), "hello").unwrap();
    fs::write(format!("{}/file2.txt", base), "world").unwrap();
    fs::write(format!("{}/sub1/file3.txt", base), "nested").unwrap();
}

// ==================== new ====================

#[test]
fn test_new_existing_dir() {
    let path = create_test_dir("new_existing");
    fs::create_dir_all(&path).unwrap();

    let dir = Dir::new(&path);
    assert!(dir.exist);
    assert!(dir.is_dir());
    assert!(dir.err.is_none());
    assert_eq!(dir.full_path, path);

    cleanup(&path);
}

#[test]
fn test_new_non_existing_dir() {
    let dir = Dir::new("/tmp/aid_nonexistent_dir_xyz");
    assert!(!dir.exist);
    assert!(dir.err.is_some());
    assert_eq!(dir.err.unwrap(), "目录不存在");
}

#[test]
fn test_new_parses_dirname_and_basepath() {
    let path = create_test_dir("new_parse");
    fs::create_dir_all(&path).unwrap();

    let dir = Dir::new(&path);
    assert!(!dir.dirname.is_empty());
    assert!(!dir.base_path.is_empty());

    cleanup(&path);
}

// ==================== new_or_create ====================

#[test]
fn test_new_or_create_existing() {
    let path = create_test_dir("noc_existing");
    fs::create_dir_all(&path).unwrap();

    let result = Dir::new_or_create(&path);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert!(dir.exist);

    cleanup(&path);
}

#[test]
fn test_new_or_create_new() {
    let path = create_test_dir("noc_new");

    let result = Dir::new_or_create(&path);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert!(dir.exist);
    assert!(Path::new(&path).exists());

    cleanup(&path);
}

// ==================== list / list_files / list_dirs ====================

#[test]
fn test_list_files_and_dirs() {
    let path = create_test_dir("list");
    setup_test_tree(&path);

    let dir = Dir::new(&path);
    assert!(dir.err.is_none());
    assert_eq!(dir.list_files().len(), 2); // file1.txt, file2.txt
    assert_eq!(dir.list_dirs().len(), 1);  // sub1

    cleanup(&path);
}

#[test]
fn test_count() {
    let path = create_test_dir("count");
    setup_test_tree(&path);

    let dir = Dir::new(&path);
    assert_eq!(dir.count(), 3); // 2 files + 1 dir

    cleanup(&path);
}

#[test]
fn test_is_empty() {
    let path = create_test_dir("empty");
    fs::create_dir_all(&path).unwrap();

    let dir = Dir::new(&path);
    assert!(dir.is_empty());

    // 添加文件后不再为空
    fs::write(format!("{}/test.txt", path), "data").unwrap();
    let dir2 = Dir::new(&path);
    assert!(!dir2.is_empty());

    cleanup(&path);
}

// ==================== up ====================

#[test]
fn test_up() {
    let path = create_test_dir("up");
    fs::create_dir_all(&path).unwrap();

    let dir = Dir::new(&path);
    let parent = dir.up();
    assert!(parent.exist);
    assert!(parent.is_dir());
    assert_ne!(parent.full_path, path);

    cleanup(&path);
}

// ==================== create ====================

#[test]
fn test_create_new_dir() {
    let path = create_test_dir("create");
    let mut dir = Dir::new(&path);
    assert!(!dir.exist);

    dir.create(0o755);
    assert!(dir.err.is_none());
    assert!(Path::new(&path).exists());

    cleanup(&path);
}

#[test]
fn test_create_existing_dir() {
    let path = create_test_dir("create_exist");
    fs::create_dir_all(&path).unwrap();

    let mut dir = Dir::new(&path);
    dir.create(0o755);
    assert!(dir.err.is_some());
    assert_eq!(dir.err.unwrap(), "目录已存在");

    cleanup(&path);
}

// ==================== remove ====================

#[test]
fn test_remove() {
    let path = create_test_dir("remove");
    setup_test_tree(&path);

    let dir = Dir::new(&path);
    let result = dir.remove();
    assert!(result.is_ok());
    assert!(!Path::new(&path).exists());
}

#[test]
fn test_remove_non_existing() {
    let dir = Dir::new("/tmp/aid_nonexistent_remove_xyz");
    let result = dir.remove();
    assert!(result.is_err());
}

// ==================== mv ====================

#[test]
fn test_mv() {
    let path = create_test_dir("mv_src");
    let dst = create_test_dir("mv_dst");
    fs::create_dir_all(&path).unwrap();
    fs::write(format!("{}/test.txt", path), "content").unwrap();

    let mut dir = Dir::new(&path);
    dir.mv(&dst);
    assert!(dir.err.is_none());
    assert!(!Path::new(&path).exists());
    assert!(Path::new(&dst).exists());
    assert_eq!(dir.full_path, dst);

    cleanup(&dst);
}

#[test]
fn test_mv_target_exists() {
    let src = create_test_dir("mv_src2");
    let dst = create_test_dir("mv_dst2");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&dst).unwrap();

    let mut dir = Dir::new(&src);
    dir.mv(&dst);
    assert!(dir.err.is_some());
    assert!(dir.err.unwrap().contains("目标目录已存在"));

    cleanup(&src);
    cleanup(&dst);
}

// ==================== zip / unzip ====================

#[test]
fn test_zip_with_lz4() {
    let src = create_test_dir("zip_src");
    let zip_file = create_test_dir("zip_out").replace("zip_out", "zip_out.lz4");
    setup_test_tree(&src);

    let dir = Dir::new(&src);
    let compressor = compressions::with_lz4();
    let result = dir.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());

    let file = result.unwrap();
    assert!(file.exist);
    assert!(file.size > 0);

    cleanup(&src);
    cleanup(&zip_file);
}

#[test]
fn test_zip_with_zlib() {
    let src = create_test_dir("zip_zlib_src");
    let zip_file = format!("{}.zlib", create_test_dir("zip_zlib_out"));
    setup_test_tree(&src);

    let dir = Dir::new(&src);
    let compressor = compressions::with_zlib();
    let result = dir.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
}

#[test]
fn test_zip_with_zstd() {
    let src = create_test_dir("zip_zstd_src");
    let zip_file = format!("{}.zstd", create_test_dir("zip_zstd_out"));
    setup_test_tree(&src);

    let dir = Dir::new(&src);
    let compressor = compressions::with_zstd();
    let result = dir.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
}

#[test]
fn test_zip_with_level() {
    let src = create_test_dir("zip_level_src");
    let zip_file = format!("{}.zstd", create_test_dir("zip_level_out"));
    setup_test_tree(&src);

    let dir = Dir::new(&src);
    let compressor = compressions::with_zstd_level(19);
    let result = dir.zip(&zip_file, compressor.as_ref());
    assert!(result.is_ok());

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
}

#[test]
fn test_zip_non_existing_dir() {
    let dir = Dir::new("/tmp/aid_nonexistent_zip_xyz");
    let compressor = compressions::with_lz4();
    let result = dir.zip("/tmp/out.lz4", compressor.as_ref());
    assert!(result.is_err());
}

#[test]
fn test_unzip_with_lz4() {
    let src = create_test_dir("unzip_src");
    let zip_file = format!("{}.lz4", create_test_dir("unzip_archive"));
    let restore = create_test_dir("unzip_restore");
    setup_test_tree(&src);

    // 压缩
    let dir = Dir::new(&src);
    let compressor = compressions::with_lz4();
    dir.zip(&zip_file, compressor.as_ref()).unwrap();

    // 解压
    let compressor = compressions::with_lz4();
    let result = Dir::unzip(&zip_file, &restore, compressor.as_ref());
    assert!(result.is_ok());

    let restored = result.unwrap();
    assert!(restored.exist);
    assert!(restored.is_dir());

    // 验证文件内容
    assert_eq!(
        fs::read_to_string(format!("{}/file1.txt", restore)).unwrap(),
        "hello"
    );
    assert_eq!(
        fs::read_to_string(format!("{}/file2.txt", restore)).unwrap(),
        "world"
    );
    assert_eq!(
        fs::read_to_string(format!("{}/sub1/file3.txt", restore)).unwrap(),
        "nested"
    );

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
    cleanup(&restore);
}

#[test]
fn test_unzip_with_zlib() {
    let src = create_test_dir("unzip_zlib_src");
    let zip_file = format!("{}.zlib", create_test_dir("unzip_zlib_archive"));
    let restore = create_test_dir("unzip_zlib_restored");
    setup_test_tree(&src);

    let dir = Dir::new(&src);
    let compressor = compressions::with_zlib();
    dir.zip(&zip_file, compressor.as_ref()).unwrap();

    let compressor = compressions::with_zlib();
    let result = Dir::unzip(&zip_file, &restore, compressor.as_ref());
    assert!(result.is_ok());

    assert_eq!(
        fs::read_to_string(format!("{}/file1.txt", restore)).unwrap(),
        "hello"
    );

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
    cleanup(&restore);
}

#[test]
fn test_unzip_with_zstd() {
    let src = create_test_dir("unzip_zstd_src");
    let zip_file = format!("{}.zstd", create_test_dir("unzip_zstd_archive"));
    let restore = create_test_dir("unzip_zstd_restored");
    setup_test_tree(&src);

    let dir = Dir::new(&src);
    let compressor = compressions::with_zstd();
    dir.zip(&zip_file, compressor.as_ref()).unwrap();

    let compressor = compressions::with_zstd();
    let result = Dir::unzip(&zip_file, &restore, compressor.as_ref());
    assert!(result.is_ok());

    assert_eq!(
        fs::read_to_string(format!("{}/sub1/file3.txt", restore)).unwrap(),
        "nested"
    );

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
    cleanup(&restore);
}

#[test]
fn test_zip_unzip_preserves_structure() {
    let src = create_test_dir("zip_struct_src");
    let zip_file = format!("{}.lz4", create_test_dir("zip_struct_archive"));
    let restore = create_test_dir("zip_struct_restored");

    // 创建更深的目录结构
    fs::create_dir_all(format!("{}/a/b/c", src)).unwrap();
    fs::write(format!("{}/root.txt", src), "root").unwrap();
    fs::write(format!("{}/a/a.txt", src), "aaa").unwrap();
    fs::write(format!("{}/a/b/b.txt", src), "bbb").unwrap();
    fs::write(format!("{}/a/b/c/c.txt", src), "ccc").unwrap();

    let dir = Dir::new(&src);
    let compressor = compressions::with_lz4();
    dir.zip(&zip_file, compressor.as_ref()).unwrap();

    let compressor = compressions::with_lz4();
    Dir::unzip(&zip_file, &restore, compressor.as_ref()).unwrap();

    assert_eq!(fs::read_to_string(format!("{}/root.txt", restore)).unwrap(), "root");
    assert_eq!(fs::read_to_string(format!("{}/a/a.txt", restore)).unwrap(), "aaa");
    assert_eq!(fs::read_to_string(format!("{}/a/b/b.txt", restore)).unwrap(), "bbb");
    assert_eq!(fs::read_to_string(format!("{}/a/b/c/c.txt", restore)).unwrap(), "ccc");

    cleanup(&src);
    let _ = fs::remove_file(&zip_file);
    cleanup(&restore);
}

#[test]
fn test_unzip_non_existing_file() {
    let compressor = compressions::with_lz4();
    let result = Dir::unzip("/tmp/aid_nonexistent.lz4", "/tmp/aid_restore", compressor.as_ref());
    assert!(result.is_err());
    match result {
        Err(e) => assert!(e.contains("读取压缩文件失败")),
        Ok(_) => panic!("期望返回错误"),
    }
}

// ==================== set_mode ====================

#[test]
#[cfg(unix)]
fn test_set_mode() {
    let path = create_test_dir("set_mode");
    fs::create_dir_all(&path).unwrap();

    let dir = Dir::new(&path);
    let result = dir.set_mode(0o700);
    assert!(result.is_ok());

    // 验证权限
    let metadata = fs::metadata(&path).unwrap();
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);

    cleanup(&path);
}

#[test]
#[cfg(unix)]
fn test_set_mode_non_existing() {
    let dir = Dir::new("/tmp/aid_nonexistent_mode_xyz");
    let result = dir.set_mode(0o755);
    assert!(result.is_err());
}