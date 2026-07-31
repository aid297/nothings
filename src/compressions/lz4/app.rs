use lz4::{Decoder, EncoderBuilder};
use std::io::{Error, Read, Write};
use crate::compressions::Compressor;

/// LZ4 压缩工具
pub struct Lz4 {
    level: u32,
}

impl Lz4 {
    /// 创建 LZ4 压缩器（默认级别 1）
    pub fn new() -> Self {
        Lz4 { level: 1 }
    }

    /// 创建 LZ4 压缩器（指定级别 1-16）
    pub fn with_level(level: u32) -> Self {
        Lz4 { level }
    }

    /// 压缩数据（默认压缩级别）
    ///
    /// # 示例
    /// ```
    /// use aid::compressions::lz4::app::Lz4;
    ///
    /// let data = b"hello world";
    /// let compressed = Lz4::compress(data).unwrap();
    /// let decompressed = Lz4::decompress(&compressed).unwrap();
    /// assert_eq!(decompressed, data);
    /// ```
    pub fn compress(data: &[u8]) -> Result<Vec<u8>, Error> {
        Self::compress_with_level(data, 1)
    }

    /// 使用指定压缩级别压缩数据
    ///
    /// # 参数
    /// - `data`: 待压缩数据
    /// - `level`: 压缩级别，范围 1-16（1 最快，16 最高压缩率）
    ///
    /// # 错误
    /// 如果压缩级别无效或压缩失败，返回 `std::io::Error`
    ///
    /// # 示例
    /// ```
    /// use aid::compressions::lz4::app::Lz4;
    ///
    /// let data = b"hello world";
    /// let compressed = Lz4::compress_with_level(data, 9).unwrap();
    /// let decompressed = Lz4::decompress(&compressed).unwrap();
    /// assert_eq!(decompressed, data);
    /// ```
    pub fn compress_with_level(data: &[u8], level: u32) -> Result<Vec<u8>, Error> {
        let mut encoder = EncoderBuilder::new()
            .level(level)
            .build(Vec::new())?;
        encoder.write_all(data)?;
        let (output, result) = encoder.finish();
        result.map(|_| output)
    }

    /// 解压数据
    ///
    /// # 错误
    /// 如果数据不是有效的 LZ4 压缩格式，返回 `std::io::Error`
    ///
    /// # 示例
    /// ```
    /// use aid::compressions::lz4::app::Lz4;
    ///
    /// let data = b"hello world";
    /// let compressed = Lz4::compress(data).unwrap();
    /// let decompressed = Lz4::decompress(&compressed).unwrap();
    /// assert_eq!(decompressed, data);
    /// ```
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut decoder = Decoder::new(data)?;
        let mut output = Vec::new();
        decoder.read_to_end(&mut output)?;
        Ok(output)
    }

    /// 压缩文件（默认压缩级别）
    ///
    /// # 参数
    /// - `input_path`: 输入文件路径
    /// - `output_path`: 输出文件路径（压缩后）
    ///
    /// # 错误
    /// 如果文件读写失败，返回 `std::io::Error`
    pub fn compress_file(input_path: &str, output_path: &str) -> Result<(), Error> {
        Self::compress_file_with_level(input_path, output_path, 1)
    }

    /// 使用指定压缩级别压缩文件
    ///
    /// # 参数
    /// - `input_path`: 输入文件路径
    /// - `output_path`: 输出文件路径（压缩后）
    /// - `level`: 压缩级别，范围 1-16（1 最快，16 最高压缩率）
    ///
    /// # 错误
    /// 如果文件读写失败，返回 `std::io::Error`
    pub fn compress_file_with_level(
        input_path: &str,
        output_path: &str,
        level: u32,
    ) -> Result<(), Error> {
        let data = std::fs::read(input_path)?;
        let compressed = Self::compress_with_level(&data, level)?;
        std::fs::write(output_path, compressed)?;
        Ok(())
    }

    /// 解压文件
    ///
    /// # 参数
    /// - `input_path`: 输入文件路径（压缩文件）
    /// - `output_path`: 输出文件路径（解压后）
    ///
    /// # 错误
    /// 如果文件读写失败或数据格式无效，返回 `std::io::Error`
    pub fn decompress_file(input_path: &str, output_path: &str) -> Result<(), Error> {
        let data = std::fs::read(input_path)?;
        let decompressed = Self::decompress(&data)?;
        std::fs::write(output_path, decompressed)?;
        Ok(())
    }

    /// 获取压缩率（压缩后大小 / 原始大小）
    ///
    /// # 返回
    /// 返回 `f64`，值越小表示压缩率越高。如果原始数据为空，返回 `0.0`
    pub fn compression_ratio(original: &[u8], compressed: &[u8]) -> f64 {
        if original.is_empty() {
            return 0.0;
        }
        compressed.len() as f64 / original.len() as f64
    }
}

impl Default for Lz4 {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for Lz4 {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        Lz4::compress_with_level(data, self.level)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        Lz4::decompress(data)
    }
}
