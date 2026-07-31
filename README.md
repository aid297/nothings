# aid

一个通用的 Rust 增强工具库，提供 `AnySlice<T>`（增强版 `Vec<T>`）、`AnyMap<K, V>`（有序键值对映射）、`Lz4`（LZ4 压缩/解压）、`Lzo`（LZO 压缩/解压）、`Zlib`（zlib 压缩/解压）、`Zstd`（Zstandard 压缩/解压）、`Bcrypt`（密码哈希与验证）、`Md5`（MD5 哈希）以及 `Sha256`（SHA256 哈希）等模块，支持丰富的链式操作方法，让数据处理更加简洁高效。

## 安装

```toml
[dependencies]
aid = "0.1.0"
```

## 模块目录

| 模块 | 说明 | 文档 |
|------|------|------|
| **AnySlice** | 增强版 `Vec<T>`，提供 43 个方法，支持链式调用、集合运算、pluck 等 | [查看文档](src/anySlices/README.md) |
| **AnyMap** | 有序键值对映射，基于平行 AnySlice 实现，保留插入顺序 | [查看文档](src/anyMaps/README.md) |
| **Lz4** | LZ4 压缩/解压，支持 1-16 级压缩级别，支持内存数据和文件 | [查看文档](src/compressions/lz4/README.md) |
| **Lzo** | LZO 压缩/解压，注重解压速度，标准 LZO1X 格式，支持内存数据和文件 | [查看文档](src/compressions/lzo/README.md) |
| **Zlib** | zlib 压缩/解压，支持 0-9 级压缩级别，通用格式广泛兼容 | [查看文档](src/compressions/zlib/README.md) |
| **Zstd** | Zstandard 压缩/解压，支持 1-21 级压缩级别，压缩率和速度最佳平衡 | [查看文档](src/compressions/zstd4/README.md) |
| **Bcrypt** | 密码哈希与验证，基于 bcrypt 2b 算法，支持自定义 cost | [查看文档](src/digests/README.md) |
| **Md5** | MD5 哈希计算，输出 32 位十六进制字符串，适用于数据完整性校验 | [查看文档](src/digests/README.md) |
| **Sha256** | SHA256 哈希计算，输出 64 位十六进制字符串，适用于安全场景 | [查看文档](src/digests/README.md) |

## 快速开始

### AnySlice — 增强版 Vec

```rust
use aid::anySlices::app::AnySlice;

let mut slice = AnySlice::new(vec![1, 2, 3, 4, 5]);

// 查询
assert!(slice.has(&3));
assert_eq!(slice.first(), Some(&1));
assert_eq!(slice.last(), Some(&5));

// 链式操作
slice.push(6).push(7);

// 集合运算
let a = AnySlice::new(vec![1, 2, 3]);
let b = vec![2, 3, 4];
let union = a.union(&b);         // [1, 2, 3, 4]
let intersection = a.intersection(&b); // [2, 3]
let difference = a.difference(&b);     // [1]
```

### AnyMap — 有序键值对映射

```rust
use aid::anyMaps::app::AnyMap;

// 从迭代器创建
let mut map = AnyMap::from_iter(vec![("alice", 90), ("bob", 85), ("charlie", 92)]);

// 查询
assert_eq!(*map.get_value_by_key(&"alice").unwrap(), 90);
assert!(map.has(&"bob"));
assert!(map.in_value(&85));

// 链式操作
map.push_datum("dave", 88);

// 按条件过滤
map.filter(|key, value| *value >= 90);

// 遍历转换
println!("{}", map.to_string(Some(", ")));
// 输出: alice: 90, charlie: 92
```

### Lz4 — LZ4 压缩/解压

```rust
use aid::compressions::lz4::app::Lz4;

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
```

### Lzo — LZO 压缩/解压

```rust
use aid::compressions::lzo::app::Lzo;

let data = b"hello world, this is some test data";

// 压缩（标准 LZO1X 原始块）
let compressed = Lzo::compress(data).unwrap();
// 解压时需要传入原始数据长度
let decompressed = Lzo::decompress(&compressed, data.len()).unwrap();
assert_eq!(decompressed, data);

// 文件压缩/解压（自动处理长度头部）
Lzo::compress_file("input.txt", "output.lzo").unwrap();
Lzo::decompress_file("output.lzo", "output.txt").unwrap();
```

### Zlib — zlib 压缩/解压

```rust
use aid::compressions::zlib::app::Zlib;

let data = b"hello world, this is some test data";

// 默认压缩（级别 6，平衡模式）
let compressed = Zlib::compress(data).unwrap();
let decompressed = Zlib::decompress(&compressed).unwrap();
assert_eq!(decompressed, data);

// 指定压缩级别（0-9，9 最高压缩率）
let compressed = Zlib::compress_with_level(data, 9).unwrap();

// 文件压缩/解压
Zlib::compress_file("input.txt", "output.zlib").unwrap();
Zlib::decompress_file("output.zlib", "output.txt").unwrap();
```

### Zstd — Zstandard 压缩/解压

```rust
use aid::compressions::zstd4::app::Zstd;

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
```

### Bcrypt — 密码哈希与验证

```rust
use aid::digests::bcrypt::Bcrypt;

let plaintext = "my_password";

// 使用默认 cost（12）生成哈希
let hashed = Bcrypt::new(plaintext).hash().unwrap();

// 验证密码
let is_valid = Bcrypt::new(plaintext).check(&hashed).unwrap();
assert!(is_valid);

// 自定义 cost
let mut bcrypt = Bcrypt::new(plaintext);
bcrypt.set_cost(10);
let hashed = bcrypt.hash().unwrap();
```

### Md5 — MD5 哈希

```rust
use aid::digests::md5::Md5Hash;

let data = "hello world";
let hashed = Md5Hash::new(data).hash();
println!("{}", hashed); // 5eb63bbbe01eeed093cb22bb8f5acdc3
```

### Sha256 — SHA256 哈希

```rust
use aid::digests::sha::Sha256Hash;

let data = "hello world";
let hashed = Sha256Hash::new(data).hash();
println!("{}", hashed); // b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
```

## License

MIT
