# nothings

一个实用的 Rust 工具库。核心是 `AnySlice<T>`（增强版 `Vec<T>`）与 `AnyMap<K, V>`（有序键值对映射），另外提供 LZ4 / zlib / Zstandard 压缩、Bcrypt / MD5 / SHA256 摘要、文件与目录操作、统一 HTTP 响应结构、时间字符串解析、声明式结构体校验、单例宏，以及面向 Go 宿主的 Wasm C-ABI 导出层。

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
| **Compressions** | `nothings::compressions` | 压缩模块总览：`Compressor` trait 与 `with_lz4()` / `with_zlib()` / `with_zstd()` 等 6 个工厂函数，用于把压缩算法作为参数传递 | [README](src/compressions/README.md) |
| **Lz4** | `nothings::compressions::lz4::app` | LZ4 frame 压缩/解压，纯 Rust 实现，可用于 wasm | [README](src/compressions/lz4/README.md) |
| **Zlib** | `nothings::compressions::zlib::app` | zlib 压缩/解压，0-9 级压缩级别 | [README](src/compressions/zlib/README.md) |
| **Zstd** | `nothings::compressions::zstd4::app` | Zstandard 压缩/解压，1-21 级压缩级别（非 wasm） | [README](src/compressions/zstd4/README.md) |
| **Digests** | `nothings::digests` | `Bcrypt` 密码哈希与验证、`Md5Hash`、`Sha256Hash` | [README](src/digests/README.md) |
| **Filesystem** | `nothings::filesystem` | `File` / `Dir`：读写、移动、删除、权限设置、整目录打包/还原（非 wasm） | [README](src/filesystem/README.md) |
| **HttpResponse** | `nothings::http_responses` | 可序列化的统一响应结构 `HttpResponse<T>`（`ok` / `bad_request` 等 7 个构造器） | [README](src/http_responses/README.md) |
| **Texts / Timer** | `nothings::texts::timer` | 将 `"1h30m"`、`"2w3d"` 等字符串解析为 `std::time::Duration` | [README](src/texts/README.md) |
| **Validations** | `nothings::validations` | 基于 `#[derive(Nothings)]` 的声明式结构体字段校验 | [README](src/validations/README.md) |
| **Singletons** | `nothings::impl_singleton!` | 通过宏为任意类型生成 `instance()` / `default()` / `with(...)` 单例接口 ⚠️ 当前外部 crate 不可用 | [README](src/singletons/README.md) |
| **Wasm 导出层** | `nothings::wasm_exports` | 13 个 `extern "C"` 导出函数，供 Go（wazero / wasmtime-go）等宿主按「指针 + 长度」调用 | [README](src/wasm_exports/README.md) |

### 暂无独立文档的模块

| 模块 | 导入路径 | 说明 |
|------|----------|------|
| **Coroutines** | `nothings::coroutines` | 预留模块，`mod.rs` 只有 `pub mod app;` 一行，`app.rs` 为 0 字节，当前没有任何内容 |

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

### Filesystem — 文件与目录

```rust
#[cfg(not(target_arch = "wasm32"))]
{
    use nothings::filesystem::file::File;
    use nothings::filesystem::dir::Dir;

    // File：写入是覆盖，不是追加
    let mut f = File::new("note.txt");
    f.write(b"hello").unwrap();
    assert_eq!(f.read().unwrap(), b"hello");
    assert_eq!(f.ext, "txt");

    // mv 不返回 Result，失败信息写进公开的 err 字段，必须手动检查
    let mut f = File::new("note.txt");
    f.mv("moved.txt");
    assert!(f.err.is_none());

    // Dir::new 会递归扫描整棵子树，大目录慎用
    let d = Dir::new_or_create("/tmp/demo").unwrap();
    println!("{} 个条目，空={}", d.count(), d.is_empty());
}
```

⚠️ `Dir::zip` 产出的**不是标准 ZIP 文件**（自定义 TLV 归档 + 压缩，magic 不是 `PK\x03\x04`），只能用 `Dir::unzip` 配同一算法还原；空目录不会被保存。`Dir::join` 当前是空操作。详见 [src/filesystem/README.md](src/filesystem/README.md)。

### HttpResponse — 统一响应结构

```rust
use nothings::http_responses::HttpResponse;

let ok: HttpResponse<i32> = HttpResponse::ok(None).content(42);
assert_eq!(ok.code, 200);
assert_eq!(serde_json::to_string(&ok).unwrap(), r#"{"code":200,"msg":"OK","content":42}"#);

let err: HttpResponse<()> = HttpResponse::un_authorization(Some("请先登录"));
assert_eq!(err.code, 401);
```

7 个构造器覆盖 200 / 201 / 202 / 204 / 400 / 401 / 500。只派生了 `Serialize`，**没有 `Debug` / `Clone` / `Deserialize`**，且状态码无法自定义（初始化入口 `empty()` 是私有的，只能靠直接改 `pub` 字段绕过）。详见 [src/http_responses/README.md](src/http_responses/README.md)。

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

### Wasm 导出层 — 供 Go 等宿主调用

```bash
cargo build --target wasm32-wasip1 --release     # 约 190 KB
```

13 个 `extern "C"` 导出函数分三组：

| 分组 | 导出函数 |
|------|----------|
| 内存管理 | `nothings_alloc`、`nothings_free` |
| 摘要与密码 | `nothings_md5`、`nothings_sha256`、`nothings_bcrypt_hash`、`nothings_bcrypt_verify` |
| 保序映射 | `nothings_anymap_new` / `insert` / `get` / `remove` / `to_json` / `len` / `drop` |

变长返回值（bcrypt 哈希、JSON）是「**前 4 字节小端长度前缀 + 数据**」的堆指针，宿主读完后必须按 `nothings_free(ptr, 4 + 长度)` 释放。定长返回值（MD5 的 32 字节、SHA256 的 64 字节）由宿主预分配缓冲区，**Rust 侧不校验长度**。

导出层在原生平台同样参与编译，可直接 `cargo test wasm_exports` 验证。完整 ABI 约定、返回码表与 Go(wazero) 示例见 [src/wasm_exports/README.md](src/wasm_exports/README.md)。

### Singletons — 当前不可用

`impl_singleton!` 宏依赖 `pub(crate)` 的 `Singleton` trait，在外部 crate 调用会直接编译失败（`E0603: trait 'Singleton' is private`），且**整个宏失效**，连 `instance()` / `default()` 也不会生成。模块内测试能过是因为同 crate。替代写法见 [src/singletons/README.md](src/singletons/README.md)。

## 已知问题

以下问题已在对应模块文档中标注，修复前请勿依赖这些方法：

| 模块 | 问题 | 影响与替代方案 |
|------|------|---------------|
| AnySlice | `remove_by_indexes` 使用惰性迭代器但未消费，实际为空操作 | 直接使 `AnyMap::filter`、`remove_by_keys`、`remove_by_values`、`remove_empty` 全部失效；改用 `to_vec()` + `filter` + `set_data` |
| AnySlice | `shuffle` 打乱的是临时克隆，返回值仍为原顺序 | 需打乱请改用 `shuffle_self` |
| AnySlice | `get_indexes_by_values` 内部 `unwrap` | 任一值不存在即 panic；`AnyMap::get_indexes_by_values` 无此问题（用 `filter_map`） |
| AnyMap | `every` 在两条分支上都返回 `&Self`，无法作为谓词 | 请改用 `to_entries().iter().all(...)` |
| Compressions | `Lz4::compress_with_level` 的 `level` 参数不影响输出 | lz4_flex 仅支持快速压缩模式，实测级别 1 / 8 / 16 输出逐字节相同；需高压缩率改用 Zstd 或 Zlib |
| Filesystem | `Dir::join` 是空操作（`PathBuf::join` 的返回值被 `let _ =` 丢弃） | 返回的仍是原目录；改用 `std::path::PathBuf::join` 自行拼接后再 `Dir::new` |
| Filesystem | `File::mv` 成功后不清除 `err` | 前一次失败的错误会一直残留，导致误判；调用前先 `f.err = None`。`Dir::mv` 无此问题 |
| Filesystem | `Dir::zip` 产出的不是标准 ZIP，且丢失空目录 | 自定义 TLV 归档 + 压缩，magic 不是 `PK\x03\x04`，系统解压工具打不开；只能用 `Dir::unzip` 配同一算法还原 |
| Filesystem | `Dir::new` 会递归扫描整棵子树 | 对大目录开销巨大且无惰性接口；只需浅层信息时直接用 `std::fs::read_dir` |
| Filesystem | `Dir::up` 对相对路径失效 | `Path::new("demo").parent()` 是空串，触发「无法返回上级目录」并原样返回自身；先 `canonicalize` |
| HttpResponse | 状态码无法自定义，`errors` 字段无实际作用 | 初始化入口 `empty()` 是私有的，只能改 `pub` 字段绕过；`std::fmt::Error` 不携带信息且被 `#[serde(skip)]` |
| HttpResponse | 没有 `Debug` / `Clone` / `PartialEq` / `Deserialize` | 不能 `{:?}` 打印、不能 `assert_eq!`、不能反序列化；只适合做服务端出参 |
| Singletons | `impl_singleton!` 在外部 crate 编译失败（`E0603: trait 'Singleton' is private`） | **整个宏失效**，`instance()` / `default()` 也不会生成；按模块文档手写 `OnceLock<Mutex<T>>` |
| Wasm 导出层 | `nothings_md5` / `nothings_sha256` 的 `out` 缓冲区无长度校验 | 宿主给小了会静默越界写坏线性内存；MD5 必须 ≥32 字节、SHA256 必须 ≥64 字节 |
| Wasm 导出层 | 导出函数无 panic 屏障 | 内部 panic（如注册表 `Mutex` 中毒）会直接 wasm trap，整个实例失效 |

## License

MIT
