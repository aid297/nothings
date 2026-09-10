# Compressions — 压缩模块总览与统一接口

`compressions` 模块提供三种压缩算法，并通过 `Compressor` trait 把它们抽象成**可以当参数传递的值**。

- 只关心「压缩一段字节」→ 直接用 `Lz4` / `Zlib` / `Zstd` 的关联函数，见各算法 README
- 需要「让用户/调用方决定用哪种算法」→ 用本页的 `Compressor` trait 与工厂函数

## 算法一览

| 算法 | 导入路径 | 级别范围 | 默认级别 | wasm | 文档 |
|------|----------|----------|----------|------|------|
| **Lz4** | `nothings::compressions::lz4::app::Lz4` | 1-16（⚠️ 不生效） | 1 | ✅ | [README](lz4/README.md) |
| **Zlib** | `nothings::compressions::zlib::app::Zlib` | 0-9 | 6 | ✅ | [README](zlib/README.md) |
| **Zstd** | `nothings::compressions::zstd4::app::Zstd` | 1-21 | 3 | ❌ | [README](zstd4/README.md) |

三者接口完全一致：`compress` / `compress_with_level` / `decompress` / `compress_file` / `decompress_file` / `compression_ratio`，全部是**关联函数**（不需要先构造实例）。

`Zstd` 依赖 C 绑定，在 `wasm32-unknown-unknown` / `wasm32-wasip1` 下整个 `zstd4` 模块不参与编译，必须用 `#[cfg(not(target_arch = "wasm32"))]` 包裹调用。

## Compressor trait

```rust
pub trait Compressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error>;
}
```

`Lz4` / `Zlib` / `Zstd` 都实现了它。实例上的 `compress` 会用**构造时记录的级别**，而关联函数 `Lz4::compress(data)` 用的是默认级别。

## 工厂函数

定义在 `nothings::compressions` 下，返回 `Box<dyn Compressor>`：

| 函数 | 签名 | 说明 |
|------|------|------|
| `with_lz4()` | `-> Box<dyn Compressor>` | LZ4，默认级别 1 |
| `with_lz4_level(level)` | `u32 -> Box<dyn Compressor>` | LZ4，级别 1-16（⚠️ 不生效，见「已知问题」） |
| `with_zlib()` | `-> Box<dyn Compressor>` | zlib，默认级别 6 |
| `with_zlib_level(level)` | `u32 -> Box<dyn Compressor>` | zlib，级别 0-9 |
| `with_zstd()` | `-> Box<dyn Compressor>` | Zstd，默认级别 3（非 wasm） |
| `with_zstd_level(level)` | `i32 -> Box<dyn Compressor>` | Zstd，级别 1-21（非 wasm） |

⚠️ `with_zstd_level` 的参数是 **`i32`**，其余两个是 `u32`。

## 快速开始

```rust
use nothings::compressions::{with_lz4, with_zlib, Compressor};

let data = b"hello world, this is some test data";

// Box<dyn Compressor> 要用 .as_ref() 转成 &dyn Compressor 再调用
let c = with_lz4();
let packed = c.compress(data).unwrap();
assert_eq!(c.decompress(&packed).unwrap(), data);

// 典型用法：把算法作为参数传进业务函数
fn pack(data: &[u8], c: &dyn Compressor) -> Vec<u8> {
    c.compress(data).unwrap()
}

assert!(!pack(data, &*with_lz4()).is_empty());
assert!(!pack(data, &*with_zlib()).is_empty());
```

也可以直接构造带级别的实例并传引用，省去 `Box`：

```rust
use nothings::compressions::zlib::app::Zlib;
use nothings::compressions::Compressor;

let z = Zlib::with_level(9);
let packed = z.compress(b"hello").unwrap();   // 走 trait 方法，级别 9
assert_eq!(z.decompress(&packed).unwrap(), b"hello");
```

## 如何选择

| 场景 | 推荐 |
|------|------|
| 追求速度、压缩率次要（缓存、网络帧） | Lz4 |
| 通用、需要跨语言/跨平台互通（HTTP `Content-Encoding: gzip` 的同族格式） | Zlib |
| 追求压缩率，或既要速度又要压缩率 | Zstd |
| wasm 环境 | Lz4 / Zlib（Zstd 不可用） |

同一份数据下的实测输出大小（输入为 200000 字节、高度重复的合成序列）：

| 算法与级别 | 输出字节数 | 相对原文 |
|-----------|-----------|---------|
| Zstd 级别 1 / 3 / 9 | 282 | 0.14% |
| Zstd 级别 19 / 21 | 283 | 0.14% |
| Zlib 级别 2-9 | 1098 | 0.55% |
| Lz4（级别 1 / 8 / 16 完全相同） | 1061 | 0.53% |
| Zlib 级别 1 | 2062 | 1.03% |
| Zlib 级别 0 | 200041 | 100.02% |

几点说明：

- **Zlib 级别 0 表示「仅存储、不压缩」**，输出会比原文更大（多出的 41 字节是 zlib 头与校验和）
- Zlib 级别 1 与级别 2 以上差异明显，级别 2 之后在本数据集上趋于一致；换一份数据分布仍可能不同，级别参数是**真实生效**的
- **Lz4 的级别参数无效**，1 / 8 / 16 输出逐字节相同，见「已知问题」
- 这份测试数据重复度极高，Zstd 的长程匹配优势被放大了；真实业务数据的排序请自行实测，不要照搬上表
- 数据很小时（几十字节）格式头开销占比极高，LZ4 输出可能大于原文，属正常现象

## 与 filesystem 的配合

`File::zip` / `File::unzip` / `Dir::zip` / `Dir::unzip` 都接收 `&dyn Compressor`，因此压缩算法可自由替换（下面示例假定磁盘上存在 `data.bin`）：

```rust
use nothings::compressions;
use nothings::filesystem::file::File;

let f = File::new("data.bin");
let packed = f.zip("data.zst", compressions::with_zstd().as_ref()).unwrap();
let _ = File::unzip("data.zst", "data.out", compressions::with_zstd().as_ref()).unwrap();
```

解压时必须用**同一种算法**，否则返回 `Err("解压失败: ...")`（各格式的 magic number 校验会失败）。

详见 [src/filesystem/README.md](../filesystem/README.md)。

## 已知问题

| 问题 | 说明 |
|------|------|
| `with_lz4_level(level)` 的 `level` 不影响输出 | 底层 lz4_flex 只实现了快速压缩模式。实测级别 1 与级别 16 的输出**逐字节相同**。参数仅为 API 兼容保留，需要更高压缩率请改用 Zstd 或 Zlib |
