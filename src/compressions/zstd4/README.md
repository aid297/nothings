# Zstd — Zstandard 压缩/解压

`Zstd` 提供基于 Zstandard (zstd) 格式的压缩/解压功能，支持 1-21 级压缩级别设置，可用于内存数据和文件。Zstandard 是 Facebook 开发的高性能压缩算法，在压缩率和速度之间提供最佳平衡。

## 快速开始

```rust
use nothings::compressions::zstd4::app::Zstd;

let data = b"hello world, this is some test data";

// 默认压缩（级别 3，平衡模式）
let compressed = Zstd::compress(data).unwrap();
let decompressed = Zstd::decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// 指定压缩级别（1-21，21 最高压缩率）
let compressed = Zstd::compress_with_level(data, 19).unwrap();

// 文件压缩/解压
Zstd::compress_file("input.txt", "output.zst").unwrap();
Zstd::decompress_file("output.zst", "output.txt").unwrap();

// 文件压缩（指定级别）
Zstd::compress_file_with_level("input.txt", "output.zst", 15).unwrap();

// 计算压缩率
let ratio = Zstd::compression_ratio(data, &compressed);
println!("压缩率: {:.2}%", ratio * 100.0);
```

## API 一览

### 内存压缩/解压

| 方法 | 说明 |
|------|------|
| `compress(data)` | 压缩数据（默认级别 3），返回 `Result<Vec<u8>, Error>` |
| `compress_with_level(data, level)` | 指定压缩级别（1-21），返回 `Result<Vec<u8>, Error>` |
| `decompress(data)` | 解压数据，返回 `Result<Vec<u8>, Error>` |
| `compression_ratio(original, compressed)` | 计算压缩率（压缩后大小 / 原始大小），返回 `f64` |

### 文件压缩/解压

| 方法 | 说明 |
|------|------|
| `compress_file(input, output)` | 压缩文件（默认级别 3） |
| `compress_file_with_level(input, output, level)` | 指定压缩级别压缩文件 |
| `decompress_file(input, output)` | 解压文件 |

### 压缩级别说明

| 级别 | 特点 |
|------|------|
| 1 | 最快压缩速度 |
| 2-3 | 平衡模式（默认级别 3） |
| 4-9 | 较高压缩率 |
| 10-15 | 高压缩率，速度较慢 |
| 16-19 | 超高压缩率 |
| 20-21 | 最高压缩率，速度最慢 |

### 与其他压缩算法对比

| 算法 | 默认级别 | 级别范围 | 特点 |
|------|---------|---------|------|
| LZ4 | 1 | 1-16 | 解压极快，压缩率适中 |
| LZO | - | - | 解压极快，压缩率适中 |
| Zlib | 6 | 0-9 | 通用格式，广泛兼容 |
| **Zstd** | **3** | **1-21** | **压缩率和速度最佳平衡** |
