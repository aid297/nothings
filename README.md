# nothings

一个实用的 Rust 工具库。核心是 `AnySlice<T>`（增强版 `Vec<T>`）与 `AnyMap<K, V>`（有序键值对映射），另外提供 LZ4 / zlib / Zstandard 压缩、Bcrypt / MD5 / SHA256 摘要、时间字符串解析、声明式结构体校验、文件与目录操作，以及面向 Go 宿主的 Wasm C-ABI 导出层。

## 安装

```toml
[dependencies]
nothings = "0.0.9"
```

- MSRV：edition 2024（需 Rust 1.85+）
- 平台：原生平台支持全部模块；`wasm32-unknown-unknown` / `wasm32-wasip1` 下 `filesystem` 与 `compressions::zstd4` 不参与编译（zstd 为 C 绑定）

## 模块目录

每个二级模块都有独立文档，下表链接到各自的 README。

| 模块 | 导入路径 | 说明 | 文档 |
|------|----------|------|------|
| **AnySlice** | `nothings::any_slices::app` | 增强版 `Vec<T>`，45 个方法：链式增删改、集合运算、`pluck`、分块等 | [README](src/any_slices/README.md) |
| **AnyMap** | `nothings::any_maps::app` | 有序键值对映射，基于平行 `AnySlice` 实现，34 个方法 | [README](src/any_maps/README.md) |
| **Lz4** | `nothings::compressions::lz4::app` | LZ4 frame 压缩/解压，纯 Rust 实现，可用于 wasm | [README](src/compressions/lz4/README.md) |
| **Zlib** | `nothings::compressions::zlib::app` | zlib 压缩/解压，0-9 级压缩级别 | [README](src/compressions/zlib/README.md) |
| **Zstd** | `nothings::compressions::zstd4::app` | Zstandard 压缩/解压，1-21 级压缩级别（非 wasm） | [README](src/compressions/zstd4/README.md) |
| **Digests** | `nothings::digests` | `Bcrypt` 密码哈希与验证、`Md5Hash`、`Sha256Hash` | [README](src/digests/README.md) |
| **Texts / Timer** | `nothings::texts::timer` | 将 `"1h30m"`、`"2w3d"` 等字符串解析为 `std::time::Duration` | [README](src/texts/README.md) |
| **Validations** | `nothings::validations` | 基于 `#[derive(Nothings)]` 的声明式结构体字段校验 | [README](src/validations/README.md) |

### 暂无独立文档的模块

| 模块 | 导入路径 | 说明 |
|------|----------|------|
| **Compressor 统一接口** | `nothings::compressions` | `Compressor` trait 与 `with_lz4()` / `with_zlib()` / `with_zstd()` 等工厂函数，用于把压缩算法作为参数传递 |
| **Filesystem** | `nothings::filesystem` | `File` / `Dir`：读写、移动、删除、权限设置、整目录 zip / unzip（非 wasm） |
| **HttpResponse** | `nothings::http_responses` | 可序列化的统一响应结构 `HttpResponse<T>`（`ok` / 错误码等构造器） |
| **Singletons** | `nothings::impl_singleton!` | 通过宏为任意类型生成 `instance()` / `default()` / `with(...)` 单例接口 |
| **Wasm 导出层** | `nothings::wasm_exports` | `extern "C"` 导出函数，供 Go（wazero / wasmtime-go）等宿主按「指针 + 长度」调用 |
| **Coroutines** | `nothings::coroutines` | 预留模块，当前为空 |

## 快速开始

下面只给最小可用示例，完整 API 见各模块 README。

### AnySlice — 增强版 Vec

```rust
use nothings::any_slices::app::AnySlice;

let mut slice = AnySlice::new(vec![1, 2, 3, 4, 5]);

// has 的参数是 &Vec<T>；first/last 返回克隆出的 Option<T>
assert!(slice.has(&vec![3]));
assert_eq!(slice.first(), Some(1));

// 链式操作
slice.push(6).push(7);

// 集合运算
let a = AnySlice::new(vec![1, 2, 3]);
let b = vec![2, 3, 4];
assert_eq!(a.union(&b).to_vec(),        &vec![1, 2, 3, 4]);
assert_eq!(a.intersection(&b).to_vec(), &vec![2, 3]);
assert_eq!(a.difference(&b).to_vec(),   &vec![1]);
```

详见 [src/any_slices/README.md](src/any_slices/README.md)。

### AnyMap — 有序键值对映射

```rust
use nothings::any_maps::app::AnyMap;

// 构造器名为 with_iter / with_hashmap
let mut map = AnyMap::with_iter(vec![("alice", 90), ("bob", 85), ("charlie", 92)]);

// get_value_by_key 返回 Result<V, Error>（克隆值，不是引用）
assert_eq!(map.get_value_by_key(&"alice").unwrap(), 90);
assert!(map.has(&"bob"));
assert!(map.in_value(&85));

map.push_datum("dave", 88);
map.each(|_key, value| value + 1);

assert_eq!(map.to_entries().len(), 4);
println!("{}", map.to_string(Some(", ")));
// alice: 91, bob: 86, charlie: 93, dave: 89
```

详见 [src/any_maps/README.md](src/any_maps/README.md)。

### 压缩 / 解压

三种算法接口一致，均为关联函数。

```rust
use nothings::compressions::lz4::app::Lz4;
use nothings::compressions::zlib::app::Zlib;

let data = b"hello world, this is some test data";

// LZ4：默认级别 1
let compressed = Lz4::compress(data).unwrap();
assert_eq!(Lz4::decompress(&compressed).unwrap(), data);

// Zlib：默认级别 6，可指定 0-9
let compressed = Zlib::compress_with_level(data, 9).unwrap();
assert_eq!(Zlib::decompress(&compressed).unwrap(), data);

// 文件压缩/解压（路径需真实存在，否则返回 Err）
Lz4::compress_file("input.txt", "output.lz4").unwrap();
Lz4::decompress_file("output.lz4", "output.txt").unwrap();
```

```rust
#[cfg(not(target_arch = "wasm32"))]
{
    use nothings::compressions::zstd4::app::Zstd;
    let compressed = Zstd::compress_with_level(b"hello", 19).unwrap();
    assert_eq!(Zstd::decompress(&compressed).unwrap(), b"hello");
}
```

也可通过统一的 `Compressor` trait 把算法作为参数传递：

```rust
use nothings::compressions::{with_lz4, with_zlib, Compressor};

fn pack(data: &[u8], c: &dyn Compressor) -> Vec<u8> {
    c.compress(data).unwrap()
}

let _ = pack(b"hello", &*with_lz4());
let _ = pack(b"hello", &*with_zlib());
```

### 摘要与密码哈希

```rust
use nothings::digests::bcrypt::Bcrypt;
use nothings::digests::md5::Md5Hash;
use nothings::digests::sha::Sha256Hash;

// Bcrypt：默认 cost 12（范围 4-31）
let plaintext = "my_password";
let hashed = Bcrypt::new(plaintext).hash().unwrap();
assert!(Bcrypt::new(plaintext).check(&hashed).unwrap());

let mut bcrypt = Bcrypt::new(plaintext);
bcrypt.set_cost(10);
let _ = bcrypt.hash().unwrap();

// MD5 → 32 位十六进制
assert_eq!(Md5Hash::new("hello world").hash(), "5eb63bbbe01eeed093cb22bb8f5acdc3");

// SHA256 → 64 位十六进制
assert_eq!(
    Sha256Hash::new("hello world").hash(),
    "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
);
```

### Timer — 时间字符串解析

```rust
use nothings::texts::timer::{Time, TimeTrait};
use std::time::Duration;

// Time 是实现 TimeTrait 的结构体，调用关联函数需把 TimeTrait 一起导入
// 注意导入路径是 texts::timer，texts::prelude 与 texts::volumer 为私有模块
let t = Time::with_text("1h30m").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(5400));
assert_eq!(t.to_chinese(), Ok("1小时30分钟".to_string()));
```

支持单位 `s` / `m` / `h` / `d` / `w`，可组合书写；详见 [src/texts/README.md](src/texts/README.md)。

### Validations — 声明式校验

```rust
use nothings::validations::check::Check;
use nothings::validations::checker::Checker;
use nothings::validations::Nothings;

#[derive(Nothings)]
struct Register {
    #[validator[rule="(required)(min>=6)(max<=20)" name="用户名" kind="string"]]
    username: String,
}

let form = Register { username: "alice".into() };

let checker = Check::new(form);
match checker.check() {
    None => println!("校验通过"),
    Some(err) => println!("校验失败: {} - {}", err.field, err.message),
}
```

规则用 `(...)` 包裹并连续书写，属性名为 `rule` / `name` / `kind` / `nested`；完整规则与操作符列表见 [src/validations/README.md](src/validations/README.md)。

## 已知问题

以下问题已在对应模块文档中标注，修复前请勿依赖这些方法：

| 问题 | 影响范围 |
|------|----------|
| `AnySlice::remove_by_indexes` 使用惰性迭代器但未消费，实际为空操作 | 直接使 `AnyMap::filter`、`remove_by_keys`、`remove_by_values`、`remove_empty` 全部失效 |
| `AnySlice::shuffle` 打乱的是临时克隆，返回值仍为原顺序 | 需打乱请改用 `shuffle_self` |
| `AnyMap::every` 在两条分支上都返回 `&Self`，无法作为谓词 | 请改用 `to_entries().iter().all(...)` |
| `AnySlice::get_indexes_by_values` 内部 `unwrap` | 任一值不存在即 panic；`AnyMap::get_indexes_by_values` 无此问题（用 `filter_map`） |
| `Lz4::compress_with_level` 的 `level` 参数不影响输出 | lz4_flex 仅支持快速压缩模式，级别参数为 API 兼容而保留 |

## License

MIT
