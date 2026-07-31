# Lz4 — LZ4 压缩/解压

`Lz4` 提供基于 LZ4 算法的高效压缩/解压功能，支持 1-16 级压缩级别设置，可用于内存数据和文件。

## 快速开始

```rust
use nothings::compressions::lz4::app::Lz4;

let data = b"hello world, this is some test data";

// 默认压缩（级别 1，最快）
let compressed = Lz4::compress(data).unwrap();
let decompressed = Lz4::decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// 指定压缩级别（1-16，16 最高压缩率）
let compressed = Lz4::compress_with_level(data, 9).unwrap();

// 文件压缩/解压
Lz4::compress_file("input.txt", "output.lz4").unwrap();
Lz4::decompress_file("output.lz4", "output.txt").unwrap();

// 文件压缩（指定级别）
Lz4::compress_file_with_level("input.txt", "output.lz4", 12).unwrap();

// 计算压缩率
let ratio = Lz4::compression_ratio(data, &compressed);
println!("压缩率: {:.2}%", ratio * 100.0);
```

## API 一览

### 内存压缩/解压

| 方法 | 说明 |
|------|------|
| `compress(data)` | 压缩数据（默认级别 1），返回 `Result<Vec<u8>, Error>` |
| `compress_with_level(data, level)` | 指定压缩级别（1-16），返回 `Result<Vec<u8>, Error>` |
| `decompress(data)` | 解压数据，返回 `Result<Vec<u8>, Error>` |
| `compression_ratio(original, compressed)` | 计算压缩率（压缩后大小 / 原始大小），返回 `f64` |

### 文件压缩/解压

| 方法 | 说明 |
|------|------|
| `compress_file(input, output)` | 压缩文件（默认级别 1） |
| `compress_file_with_level(input, output, level)` | 指定压缩级别压缩文件 |
| `decompress_file(input, output)` | 解压文件 |

### 压缩级别说明

| 级别 | 特点 |
|------|------|
| 1 | 最快压缩速度（默认） |
| 2-5 | 平衡模式 |
| 6-9 | 较高压缩率 |
| 10-16 | 最高压缩率（HC 模式），速度较慢 |
