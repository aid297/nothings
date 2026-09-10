# Wasm Exports — C-ABI 导出层

`wasm_exports` 模块把库里的摘要、密码哈希和保序映射能力包装成一组 `extern "C"` 函数，供 Go（wazero / wasmtime-go）等 Wasm 宿主按「指针 + 长度」的方式直接调用。

模块在**原生平台也参与编译**，所以可以直接用 `cargo test` 验证导出逻辑，不必每次构建 wasm。

## 构建

```bash
# wasip1（wazero / wasmtime-go 等 WASI 宿主）
cargo build --target wasm32-wasip1 --release
# 产物：target/wasm32-wasip1/release/nothings.wasm，约 190 KB

# 浏览器
cargo build --target wasm32-unknown-unknown --release
# 产物：target/wasm32-unknown-unknown/release/nothings.wasm，约 178 KB
```

两个目标均已实测可成功构建。wasip1 产物导出表共 14 项：13 个函数 + 1 个 `memory`。

`Cargo.toml` 里 `crate-type = ["lib", "cdylib"]`，`cdylib` 是产出 `.wasm` 的关键。浏览器目标下 `getrandom` 的 `wasm_js` feature 已配好（随机数走 JS 宿主），wasip1 下走 WASI `random_get`，无需额外配置。

## 导出清单

| 导出名 | 签名 | 用途 |
|--------|------|------|
| `memory` | — | Wasm 线性内存，宿主读写数据都要经过它 |
| `nothings_alloc` | `(len: usize) -> *mut u8` | 在 Wasm 内存里分配输入缓冲区 |
| `nothings_free` | `(ptr: *mut u8, len: usize)` | 释放 `alloc` 或变长返回值分配的内存 |
| `nothings_md5` | `(data: *const u8, len: usize, out: *mut u8) -> i32` | MD5，32 字节 hex 写入 `out` |
| `nothings_sha256` | `(data: *const u8, len: usize, out: *mut u8) -> i32` | SHA256，64 字节 hex 写入 `out` |
| `nothings_bcrypt_hash` | `(password: *const u8, len: usize, cost: u32) -> *mut u8` | bcrypt 哈希 |
| `nothings_bcrypt_verify` | `(password: *const u8, plen: usize, hash: *const u8, hlen: usize) -> i32` | bcrypt 校验 |
| `nothings_anymap_new` | `() -> u32` | 创建保序映射，返回句柄 |
| `nothings_anymap_insert` | `(h: u32, key: *const u8, klen: usize, val: *const u8, vlen: usize) -> i32` | 插入/更新（值为 JSON 文本） |
| `nothings_anymap_get` | `(h: u32, key: *const u8, klen: usize) -> *mut u8` | 读取单个值 |
| `nothings_anymap_remove` | `(h: u32, key: *const u8, klen: usize) -> i32` | 删除单个 key |
| `nothings_anymap_to_json` | `(h: u32) -> *mut u8` | 导出整个映射为保序 JSON 对象 |
| `nothings_anymap_len` | `(h: u32) -> i32` | 键值对数量 |
| `nothings_anymap_drop` | `(h: u32) -> i32` | 销毁句柄，释放 Rust 侧内存 |

`usize` 在 wasm32 上是 4 字节，宿主侧按 `i32` / `uint32` 传即可。

## ABI 约定

### 输入

所有输入都是 `ptr: *const u8` + `len: usize` 的成对参数。**空指针或零长度会被当成空输入**（`slice_from_raw` 返回 `&[]`），不会崩溃：

```rust
unsafe fn slice_from_raw<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if ptr.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(ptr, len) } }
}
```

后果：`nothings_md5(null, 0, out)` 返回的是**空字符串的 MD5**（`d41d8cd98f00b204e9800998ecf8427e`），而不是错误码。宿主侧要区分「没传」和「传了空串」得自己判断。

### 定长返回值

`nothings_md5` / `nothings_sha256` 由**宿主预分配**输出缓冲区，通过 `out` 指针写入：

| 函数 | `out` 最小字节数 | 内容 |
|------|-----------------|------|
| `nothings_md5` | 32 | 小写十六进制 ASCII，**不含**结尾 `\0` |
| `nothings_sha256` | 64 | 同上 |

⚠️ Rust 侧用 `copy_nonoverlapping` 直接写，**不做长度校验**。缓冲区给小了就是越界写，会踩坏线性内存且不报错——这是宿主的责任。

### 变长返回值

`nothings_bcrypt_hash` / `nothings_anymap_get` / `nothings_anymap_to_json` 返回 `*mut u8`，格式是：

```text
┌────────────────────┬──────────────────────┐
│ 前 4 字节：小端 u32 │ 其后 N 字节：数据本体 │
│ = 数据长度 N        │                      │
└────────────────────┴──────────────────────┘
```

失败返回**空指针**。宿主读完后必须释放：

```text
nothings_free(ptr, 4 + N)
```

`len` 必须是 `4 + N`，不是 `N`——`free` 内部用 `Vec::from_raw_parts(ptr, len, len)` 还原，长度必须与分配时的容量一致，传错就是未定义行为。

### 返回码

`0` 或正数表示成功，负数表示失败。各函数的具体取值见下表。

## 函数详解

### `nothings_alloc` / `nothings_free`

```text
nothings_alloc(len) -> *mut u8     len == 0 时返回空指针
nothings_free(ptr, len)            ptr 为空或 len == 0 时是空操作
```

`alloc` 返回的缓冲区 `len` 等于容量，释放时传同一个 `len`。

典型宿主流程：`alloc(n)` → 把输入写进线性内存 → 调业务函数 → `free`。

### `nothings_md5` / `nothings_sha256`

| 返回码 | 含义 |
|--------|------|
| `0` | 成功 |
| `-1` | 输入不是合法 UTF-8 |
| `-2` | `out` 是空指针 |

⚠️ 这两个函数**先做 UTF-8 校验再检查 `out`**，所以 `out` 为空但输入非法时返回的是 `-1` 而不是 `-2`。

底层 [`Md5Hash`](../digests/README.md) / `Sha256Hash` 的入参是 `&str`，因此接口只接受文本；要对任意二进制求摘要，得先把它编成合法 UTF-8（例如 base64），或者改用原生 Rust API。

### `nothings_bcrypt_hash` / `nothings_bcrypt_verify`

```text
nothings_bcrypt_hash(password, len, cost) -> *mut u8   // 长度前缀指针，约 60 字节
nothings_bcrypt_verify(password, plen, hash, hlen) -> i32
```

`nothings_bcrypt_hash` 失败（非 UTF-8，或 `cost` 超出 4-31）返回**空指针**，没有返回码可查，只能判空。

| `verify` 返回码 | 含义 |
|----------------|------|
| `1` | 密码匹配 |
| `0` | 密码不匹配 |
| `-1` | 输入非法：password 或 hash 非 UTF-8，或 bcrypt 解析失败 |

注意 `1` / `0` / `-1` 三态，**不能**简单当成 C 风格的「非 0 即真」来用——`-1` 也是非 0。

`cost` 直接透传给 [`Bcrypt::set_cost`](../digests/README.md)。cost 每加 1，耗时翻倍；宿主侧要注意别把高 cost 的调用放在请求热路径上。

### AnyMap 系列

Go 的 `map` 本身无序，这组导出提供**保持插入顺序**的键值映射，值统一是 JSON。

Rust 侧用一个全局注册表存实例，宿主持有 `u32` 句柄：

```rust
static ANYMAP_REGISTRY: Mutex<Option<HashMap<u32, AnyMap<String, serde_json::Value>>>> = Mutex::new(None);
static ANYMAP_NEXT_HANDLE: AtomicU32 = AtomicU32::new(1);   // 0 保留为无效值
```

| 函数 | 返回码 |
|------|--------|
| `nothings_anymap_new` | 句柄（从 1 递增）；实际不会失败，**永远不会返回 0** |
| `nothings_anymap_insert` | `0` 成功（key 已存在则原地更新、**保持原位置**）／`-1` 句柄无效／`-2` key 非 UTF-8 或 value 不是合法 JSON |
| `nothings_anymap_get` | 长度前缀指针；key 不存在或句柄无效返回空指针 |
| `nothings_anymap_remove` | `1` 删除成功／`0` key 不存在／`-1` 句柄无效或 key 非 UTF-8 |
| `nothings_anymap_to_json` | 长度前缀指针（形如 `{"a":1,"b":2}`）；句柄无效返回空指针 |
| `nothings_anymap_len` | 键值对数量；句柄无效返回 `-1` |
| `nothings_anymap_drop` | `0` 成功／`-1` 句柄无效 |

几点说明：

- **值必须是 JSON 文本**，不是裸字符串。插字符串要传 `"alice"`（带引号），传 `alice` 会得到 `-2`
- `insert` 的「查 key 是否存在 + 写入」在**同一次加锁内**完成，多线程宿主下是原子的
- `to_json` 手工拼接 JSON，但 key 和 value 都经过 `serde_json::to_string`，转义是正确的
- 句柄只增不复用；`drop` 之后再拿旧句柄调用一律返回 `-1` / 空指针
- 注册表是**进程级全局**的。同一个 runtime 里加载多份 wasm 模块，各自有独立注册表；但一个模块内的所有调用共享同一张表
- `nothings_anymap_len` 返回 `i32`，映射超过 21 亿项会溢出——实际不会遇到
- `insert` 源码里还有一个文档未列出的 `-3`（`set_value_by_key` 失败）。因为前面已经用 `has` 判过 key 存在，**这个分支实际不可达**

## Go 宿主示例（wazero）

```text
ctx := context.Background()
runtime := wazero.NewRuntime(ctx)
defer runtime.Close(ctx)

// 1. 实例化模块
cfg := wazero.NewModuleConfig().WithSysWalltime()
mod, err := runtime.InstantiateWithConfig(ctx, wasmBin, cfg)
if err != nil { log.Fatal(err) }

mem := mod.Memory()

// 2. 写入输入：alloc -> 写内存
alloc := mod.ExportedFunction("nothings_alloc")
free  := mod.ExportedFunction("nothings_free")
md5Fn := mod.ExportedFunction("nothings_md5")

input := []byte("hello")
res, _ := alloc.Call(ctx, uint64(len(input)))
ptr := uint32(res[0])
mem.Write(ptr, input)

// 3. 准备输出缓冲区并调用
outPtr := uint32(len(input))   // 紧接输入之后，实际项目请再 alloc 一块
var out [32]byte
_ = out
code, _ := md5Fn.Call(ctx, uint64(ptr), uint64(len(input)), uint64(outPtr))
if int32(code[0]) != 0 { log.Fatal("md5 failed") }

buf, _ := mem.Read(outPtr, 32)
fmt.Println(string(buf))       // 5d41402abc4b2a76b9719d911017c592

// 4. 释放
free.Call(ctx, uint64(ptr), uint64(len(input)))
```

读取变长返回值时，先读前 4 字节拿长度：

```text
get := mod.ExportedFunction("nothings_anymap_get")
res, _ := get.Call(ctx, uint64(handle), uint64(kPtr), uint64(kLen))
p := uint32(res[0])
if p == 0 { /* 不存在或句柄无效 */ }

head, _ := mem.Read(p, 4)
n := binary.LittleEndian.Uint32(head)
body, _ := mem.Read(p+4, n)
free.Call(ctx, uint64(p), uint64(4+n))   // 注意是 4+n
```

上面的 Go 片段是**示意**，错误处理被简化了；`outPtr` 在真实代码里应该单独 `alloc` 而不是复用输入后面的地址。

## 测试

导出函数在原生平台可直接调用，仓库自带的测试就是这么做的：

```bash
cargo test wasm_exports
```

覆盖了 MD5 / SHA256 的正常与非 UTF-8 路径、bcrypt 往返、以及 AnyMap 的完整生命周期（插入、读取、保序导出、原地更新、删除、非法 JSON、无效句柄、销毁）。

## 已知问题与注意事项

| 问题 | 说明 |
|------|------|
| `out` 缓冲区无长度校验 | `nothings_md5` / `nothings_sha256` 直接 `copy_nonoverlapping` 写 32 / 64 字节，宿主给小了会静默越界 |
| 只接受 UTF-8 输入 | 摘要与 bcrypt 都先把字节转 `&str`，二进制数据无法直接处理 |
| `nothings_free` 的 `len` 必须精确 | 内部按 `Vec::from_raw_parts(ptr, len, len)` 还原，`len` 与分配时容量不一致即未定义行为 |
| `bcrypt_hash` 无错误码 | 失败只返回空指针，无法区分「非 UTF-8」和「cost 非法」 |
| 句柄计数器理论上会回绕 | `AtomicU32::fetch_add` 在 release 下溢出回绕，创建超过 42 亿个句柄后会撞上保留值 0 并复用旧句柄。实际不可达，但长驻进程值得知道 |
| 无 panic 屏障 | 导出函数没有 `catch_unwind`。内部若 panic（例如注册表 `Mutex` 中毒后 `.lock().unwrap()`），默认行为是 wasm trap，整个实例失效 |

`compressions::zstd4` 与 `filesystem` 在 wasm 下不参与编译，因此导出层**没有**压缩和文件相关的函数——只有摘要、bcrypt 和 AnyMap 三类。

## 备注

- 模块文件是 `src/wasm_exports/mod.rs`，导出函数全部标注 `#[unsafe(no_mangle)]`（Rust 2024 edition 的写法）
- 源码里的模块级 rustdoc（`//!`）是本页的**精简版**——只写了 ABI 约定和两行 Go 片段；导出清单、逐个返回码、注意事项都只在本页有。`cargo doc --open` 看到的是那份摘要
- AnyMap 相关导出的行为细节见 [src/any_maps/README.md](../any_maps/README.md)；注意 `nothings_anymap_remove` 走的是 `remove_by_key`（单个、可用），**不受** `remove_by_keys` 失效问题的影响
