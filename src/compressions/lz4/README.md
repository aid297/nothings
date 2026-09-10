# Lz4 — LZ4 压缩/解压

`Lz4` 提供基于 LZ4 算法的高效压缩/解压功能，支持内存数据和文件压缩，输出标准 LZ4 frame 格式（与 liblz4 兼容）。基于纯 Rust 实现（lz4_flex），可用于 wasm 环境。

## 快速开始

```rust
use nothings::compressions::lz4::app::Lz4;

let data = b"hello world, this is some test data";

// 默认压缩（级别 1，最快）
let compressed = Lz4::compress(data).unwrap();
let decompressed = Lz4::decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// 指定压缩级别（参数保留，当前不影响结果）
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
| `compress_with_level(data, level)` | 指定压缩级别（参数保留，当前不影响结果），返回 `Result<Vec<u8>, Error>` |
| `decompress(data)` | 解压数据，返回 `Result<Vec<u8>, Error>` |
| `compression_ratio(original, compressed)` | 计算压缩率（压缩后大小 / 原始大小），返回 `f64` |

### 文件压缩/解压

| 方法 | 说明 |
|------|------|
| `compress_file(input, output)` | 压缩文件（默认级别 1） |
| `compress_file_with_level(input, output, level)` | 指定压缩级别压缩文件 |
| `decompress_file(input, output)` | 解压文件 |

### 压缩级别说明

当前基于 lz4_flex（纯 Rust 实现），仅实现快速压缩模式：

- `compress_with_level` / `with_level` 的级别参数（1-16）保留用于 API 兼容，不影响压缩结果
- 如需更高压缩率，建议使用 Zstd 或 Zlib 模块
