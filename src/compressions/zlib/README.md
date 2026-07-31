# Zlib — zlib 压缩/解压

`Zlib` 提供基于 zlib 格式的压缩/解压功能，支持 0-9 级压缩级别设置，可用于内存数据和文件。zlib 是最通用的压缩格式之一，广泛兼容各种工具和协议。

## 快速开始

```rust
use nothings::compressions::zlib::app::Zlib;

let data = b"hello world, this is some test data";

// 默认压缩（级别 6，平衡模式）
let compressed = Zlib::compress(data).unwrap();
let decompressed = Zlib::decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// 指定压缩级别（0-9，9 最高压缩率，0 不压缩）
let compressed = Zlib::compress_with_level(data, 9).unwrap();

// 文件压缩/解压
Zlib::compress_file("input.txt", "output.zlib").unwrap();
Zlib::decompress_file("output.zlib", "output.txt").unwrap();

// 文件压缩（指定级别）
Zlib::compress_file_with_level("input.txt", "output.zlib", 9).unwrap();

// 计算压缩率
let ratio = Zlib::compression_ratio(data, &compressed);
println!("压缩率: {:.2}%", ratio * 100.0);
```

## API 一览

### 内存压缩/解压

| 方法 | 说明 |
|------|------|
| `compress(data)` | 压缩数据（默认级别 6），返回 `Result<Vec<u8>, Error>` |
| `compress_with_level(data, level)` | 指定压缩级别（0-9），返回 `Result<Vec<u8>, Error>` |
| `decompress(data)` | 解压数据，返回 `Result<Vec<u8>, Error>` |
| `compression_ratio(original, compressed)` | 计算压缩率（压缩后大小 / 原始大小），返回 `f64` |

### 文件压缩/解压

| 方法 | 说明 |
|------|------|
| `compress_file(input, output)` | 压缩文件（默认级别 6） |
| `compress_file_with_level(input, output, level)` | 指定压缩级别压缩文件 |
| `decompress_file(input, output)` | 解压文件 |

### 压缩级别说明

| 级别 | 特点 |
|------|------|
| 0 | 不压缩，仅存储 |
| 1 | 最快压缩速度 |
| 2-5 | 平衡模式 |
| 6 | 默认级别，速度与压缩率平衡 |
| 7-8 | 较高压缩率 |
| 9 | 最高压缩率，速度较慢 |
