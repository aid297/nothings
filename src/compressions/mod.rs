pub mod lz4;
pub mod zlib;
#[cfg(not(target_arch = "wasm32"))]
pub mod zstd4;

use std::io::Error;

/// 统一压缩算法接口
pub trait Compressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
}

/// 创建 LZ4 压缩器（默认级别 1）
pub fn with_lz4() -> Box<dyn Compressor> {
    Box::new(lz4::app::Lz4::new())
}

/// 创建 LZ4 压缩器（指定级别 1-16）
pub fn with_lz4_level(level: u32) -> Box<dyn Compressor> {
    Box::new(lz4::app::Lz4::with_level(level))
}

/// 创建 Zlib 压缩器（默认级别 6）
pub fn with_zlib() -> Box<dyn Compressor> {
    Box::new(zlib::app::Zlib::new())
}

/// 创建 Zlib 压缩器（指定级别 0-9）
pub fn with_zlib_level(level: u32) -> Box<dyn Compressor> {
    Box::new(zlib::app::Zlib::with_level(level))
}

/// 创建 Zstd 压缩器（默认级别 3）
///
/// 仅在非 wasm 目标可用（zstd 为 C 绑定实现）
#[cfg(not(target_arch = "wasm32"))]
pub fn with_zstd() -> Box<dyn Compressor> {
    Box::new(zstd4::app::Zstd::new())
}

/// 创建 Zstd 压缩器（指定级别 1-21）
///
/// 仅在非 wasm 目标可用（zstd 为 C 绑定实现）
#[cfg(not(target_arch = "wasm32"))]
pub fn with_zstd_level(level: i32) -> Box<dyn Compressor> {
    Box::new(zstd4::app::Zstd::with_level(level))
}
