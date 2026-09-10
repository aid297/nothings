# Filesystem — 文件与目录操作

`filesystem` 模块提供 `File` 与 `Dir` 两个结构体，覆盖读写、移动、删除、权限设置，以及基于自定义归档格式的整目录打包/还原。

⚠️ **本模块整体不参与 wasm 编译**：`src/lib.rs` 中声明为 `#[cfg(not(target_arch = "wasm32"))] pub mod filesystem;`。在 wasm 目标下引用它会直接编译失败，需要用 `#[cfg(not(target_arch = "wasm32"))]` 包裹。

```rust
use nothings::filesystem::file::File;
use nothings::filesystem::dir::Dir;
```

## 两套错误处理风格

这是本模块最容易踩的地方——**同一个结构体上混用了两种错误返回方式**：

| 风格 | 方法 | 失败时 |
|------|------|--------|
| `Result<_, String>` | `File::write` / `read` / `remove` / `zip`，`Dir::new_or_create` / `remove` / `set_mode` / `zip` / `unzip` | 返回 `Err(中文描述)`，必须处理 |
| `err` 字段 | `File::mv`，`Dir::list` / `create` / `mv` / `up` / `join` | **返回 `&Self` 或 `Self`，不报错**，错误信息写进公开的 `err: Option<String>` 字段 |

用第二类方法后**必须手动检查 `err`**，否则失败会被静默吞掉：

```rust
use nothings::filesystem::file::File;

let mut f = File::new("a.txt");
f.mv("b.txt");
if let Some(e) = &f.err {
    println!("移动失败: {e}");   // 例如 "目标文件已存在"
}
```

## File

### 字段（全部 `pub`）

| 字段 | 类型 | 说明 |
|------|------|------|
| `full_path` | `String` | 构造时传入的完整路径 |
| `filename` | `String` | 文件名（含扩展名）；路径无文件名时为 `""` |
| `base_path` | `String` | 上级目录；无上级时为 `""` |
| `ext` | `String` | 扩展名（不含点）；无扩展名时为 `""` |
| `size` | `u64` | 字节数；文件不存在时为 `0` |
| `exist` | `bool` | **构造时**是否存在 |
| `info` | `Option<fs::FileType>` | 文件类型；不存在时为 `None` |
| `mode` | `Option<fs::Permissions>` | 权限；不存在时为 `None` |
| `err` | `Option<String>` | 由 `mv` 写入的错误信息 |

路径解析在 `File::new` 里就完成，**文件不存在也不会报错**，只是 `exist = false`、`size = 0`、`info`/`mode` 为 `None`，而 `filename` / `ext` / `base_path` 仍会正常解析出来。

### 方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `new(full_path)` | `&str -> Self` | 解析路径、读取元数据 |
| `is_file(&self)` | `-> bool` | 是否普通文件 |
| `is_dir(&self)` | `-> bool` | 是否目录 |
| `write(&mut self, data)` | `&[u8] -> Result<bool, String>` | **覆盖**写入（不是追加），成功后刷新 `size` / `exist` / `info` / `mode` |
| `read(&self)` | `-> Result<Vec<u8>, String>` | 一次性读入全部内容 |
| `remove(self)` | `-> Result<bool, String>` | 删除文件，**消耗自身** |
| `mv(&mut self, dst)` | `&str -> &Self` | 移动/重命名；目标已存在则不执行并写入 `err` |
| `zip(&self, filename, compressor)` | `&str, &dyn Compressor -> Result<File, String>` | 压缩为另一个文件，返回新 `File` |
| `unzip(filename, output_file, compressor)` | `&str, &str, &dyn Compressor -> Result<File, String>` | **关联函数**，从压缩文件还原 |

### 示例

```rust
use nothings::filesystem::file::File;

let mut f = File::new("note.txt");
f.write(b"hello").unwrap();
assert_eq!(f.read().unwrap(), b"hello");
assert_eq!(f.size, 5);
assert_eq!(f.ext, "txt");

f.remove().unwrap();     // f 在此被消耗
```

压缩/解压需要传入 [`Compressor`](../compressions/README.md)：

```rust
use nothings::compressions;
use nothings::filesystem::file::File;

let f = File::new("note.txt");
let c = compressions::with_zlib();

let packed = f.zip("note.txt.z", c.as_ref()).unwrap();
let restored = File::unzip("note.txt.z", "note.out", c.as_ref()).unwrap();

assert_eq!(restored.read().unwrap(), f.read().unwrap());
```

解压时算法必须与压缩时一致，否则返回 `Err("解压失败: WrongMagicNumber...")`。

## Dir

### 字段（全部 `pub`）

| 字段 | 类型 | 说明 |
|------|------|------|
| `full_path` | `String` | 完整路径 |
| `dirname` | `String` | 目录名 |
| `base_path` | `String` | 上级目录路径 |
| `exist` | `bool` | 构造时是否存在 |
| `info` | `Option<fs::FileType>` | 文件类型 |
| `mode` | `Option<fs::Permissions>` | 权限 |
| `sub_files` | `AnySlice<File>` | **当前层**的文件 |
| `sub_dirs` | `AnySlice<Dir>` | **当前层**的子目录（每个都已递归扫描完毕） |
| `err` | `Option<String>` | 错误信息 |

### ⚠️ `Dir::new` 会递归扫描整棵子树

`Dir::new` 内部调用 `list()`，而 `list()` 对每个子目录又会构造 `Dir::new`——**递归到底**。这意味着：

```rust
use nothings::filesystem::dir::Dir;

let d = Dir::new("/Users");   // 会走完整个 /Users 下所有层级的所有文件
```

对大目录这是灾难性的开销（每个文件的元数据都会被读取）。只想知道「当前层有什么」的话，目前**没有**惰性接口可用；如果只需要浅层信息，建议直接用 `std::fs::read_dir`。

### 方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `new(full_path)` | `&str -> Self` | 递归扫描；目录不存在时 `exist = false` 且 `err = Some("目录不存在")` |
| `new_or_create(full_path)` | `&str -> Result<Self, String>` | 不存在则 `create_dir_all` 后再构造 |
| `is_dir(&self)` | `-> bool` | 是否目录 |
| `list(&mut self)` | `-> &Self` | **重新**扫描当前层，会先清空 `sub_files` / `sub_dirs` 并清除 `err` |
| `list_files(&self)` | `-> &AnySlice<File>` | 已扫描的文件列表 |
| `list_dirs(&self)` | `-> &AnySlice<Dir>` | 已扫描的子目录列表 |
| `count(&self)` | `-> usize` | 当前层条目数 = 文件数 + 子目录数 |
| `is_empty(&self)` | `-> bool` | 当前层是否为空 |
| `up(self)` | `-> Self` | 返回上级目录；无上级时返回自身并写入 `err` |
| `join(self, paths)` | `&Vec<&str> -> Self` | ⚠️ **当前无效**，见「已知问题」 |
| `remove(self)` | `-> Result<bool, String>` | `remove_dir_all`，**递归删除全部内容**，消耗自身 |
| `create(&mut self, mode)` | `u32 -> &Self` | 创建目录（递归）；已存在则写入 `err` |
| `set_mode(&self, mode)` | `u32 -> Result<bool, String>` | 设置 Unix 权限位；非 Unix 平台**总是**返回 `Err` |
| `mv(&mut self, dst)` | `&str -> &Self` | 移动/重命名；成功后自动重新 `list()` |
| `zip(&self, filename, compressor)` | `&str, &dyn Compressor -> Result<File, String>` | 打包并压缩整个目录 |
| `unzip(filename, output_dir, compressor)` | `&str, &str, &dyn Compressor -> Result<Dir, String>` | **关联函数**，还原目录 |

`create` / `set_mode` 的 `mode` 是 Unix 权限位（如 `0o755`）。非 Unix 平台上 `create` 会**忽略** `mode`，`set_mode` 直接返回 `Err("当前平台不支持 mode 设置")`。

### 示例

```rust
use nothings::filesystem::dir::Dir;

// 用绝对路径：相对路径下 up() 会失效，见「已知问题」
let d = Dir::new_or_create("/tmp/demo").unwrap();
println!("条目数 {}，是否为空 {}", d.count(), d.is_empty());

for f in d.list_files().to_vec() {
    println!("{} ({} bytes)", f.filename, f.size);
}
for sub in d.list_dirs().to_vec() {
    println!("[dir] {}", sub.dirname);
}

let parent = d.up();
println!("上级目录: {} err: {:?}", parent.full_path, parent.err);
```

## zip / unzip 的归档格式

⚠️ **`zip` 产出的不是 ZIP 文件**。名字有误导性，实际流程是：

1. 递归收集目录下所有**文件**，读入内存
2. 按自定义 TLV 格式拼成一个字节流：`文件数(u32)` 然后对每个文件写 `路径长(u32) + 路径 + 数据长(u64) + 数据`（整数均为小端）
3. 用传入的 `Compressor` 压缩整个字节流并落盘

实测归档头 4 字节是压缩算法的 magic（LZ4 frame 为 `04 22 4D 18`），**不是** ZIP 的 `50 4B 03 04`（`PK\x03\x04`）。因此：

- 产物**无法**被系统解压工具、`unzip` 命令或任何 zip 库打开
- 只能用本模块的 `Dir::unzip` 配合**同一个压缩算法**还原
- 扩展名建议按算法命名（`.lz4` / `.zst` / `.zz`），不要写 `.zip`
- 整个目录树会先全部读进内存，**不适合大目录**
- 空目录不会被保存，还原后丢失（`collect_files` 只收集 `is_file()` 的条目）
- 压缩算法在小数据上会**放大**体积：实测 9 字节内容打包后为 89 字节

```rust
use nothings::compressions;
use nothings::filesystem::dir::Dir;

let dir = Dir::new("/tmp/assets");
let c = compressions::with_lz4();

dir.zip("/tmp/assets.lz4", c.as_ref()).unwrap();
let restored = Dir::unzip("/tmp/assets.lz4", "/tmp/assets_copy", c.as_ref()).unwrap();

// 注意：不要断言 restored.count() == dir.count()
// 若原目录含空子目录，它们不会进归档，还原后 count 会变小（实测 2 -> 1）
assert_eq!(restored.list_files().len(), dir.list_files().len());
```

`Dir::unzip` 会自动 `create_dir_all` 输出目录并还原嵌套结构。

### 安全提示

`unzip` 内部用 `output.join(relative_path)` 写文件，**不校验路径是否逃出输出目录**。本模块自己产出的归档是安全的（路径来自 `strip_prefix`），但如果归档来自不可信来源，构造 `../../etc/passwd` 这样的路径即可写到任意位置（zip-slip）。**不要对不可信归档调用 `Dir::unzip`。**

## 已知问题

### 1. `Dir::join` 是空操作

```rust
pub fn join(mut self, paths: &Vec<&str>) -> Self {
    let path = PathBuf::from(self.full_path.as_str());
    for i in 0..paths.len() {
        let _ = path.join(paths[i]);     // PathBuf::join 返回新值，不修改自身
    }
    ...
    Dir::new(path.to_str().unwrap())     // path 始终等于 full_path
}
```

`PathBuf::join` 是**取值**方法（返回新的 `PathBuf`），结果被 `let _ =` 丢弃，`path` 从未改变。实测 `Dir::new("/tmp").join(&vec!["a", "b"]).full_path == "/tmp"`。

替代写法：

```rust
use std::path::PathBuf;
use nothings::filesystem::dir::Dir;

let base = PathBuf::from("/tmp");
let target = base.join("a").join("b");
let d = Dir::new(target.to_str().unwrap());
```

### 2. `File::mv` 成功后不清除 `err`

`mv` 只在失败分支写 `err`，从不重置。一旦某次调用失败，**后续成功的调用仍会读到旧错误**：

```text
File::mv 失败后 err = Some("目标文件已存在")
File::mv 成功后 err = Some("目标文件已存在")   ← 实际已移动成功，err 却是旧值
```

替代写法：调用前先手动清空。

```rust
use nothings::filesystem::file::File;

let mut f = File::new("a.txt");
f.err = None;
f.mv("b.txt");
assert!(f.err.is_none());
```

`Dir::mv` **没有**这个问题——它在成功分支调用 `list()`，而 `list()` 会把 `err` 置为 `None`。

### 3. `set_mode` 不检查目标是不是目录

`Dir::set_mode` 只判断 `path.exists()`，不判断 `is_dir()`。对一个普通文件路径调用它会返回 `Ok(true)` 并真的改掉文件权限。

### 4. `Dir::up` 对相对路径失效

`up()` 用 `self.base_path == ""` 判断是否已到顶层，而相对路径的 `parent()` 恰好就是空串：

```text
Dir::new("demo").up()           -> full_path = "demo"      err = Some("无法返回上级目录")
Dir::new("/tmp/demo").up()      -> full_path = "/tmp"      err = None
```

即对 `"demo"` 调 `up()` 会**原样返回自己**并报错，而不是回到当前目录。需要相对路径时先自行规范化：

```rust
use std::path::Path;
use nothings::filesystem::dir::Dir;

let abs = std::fs::canonicalize("demo").unwrap();
let d = Dir::new(abs.to_str().unwrap());

// up() 接收 self 会消耗 d，所以先把期望值算出来
let expected = Path::new(&d.full_path).parent().unwrap().to_str().unwrap().to_string();
let parent = d.up();
assert_eq!(parent.full_path, expected);
```

`up()` 失败时返回的是**原目录自身**（只是多了 `err`），不是一个空 `Dir`，别把它当成「已到根」的信号。

### 5. `read` 依赖构造时的 `exist` 快照

`File::read` 先检查 `self.exist`，而 `exist` 是 `File::new` 时记录的。若文件在 `File::new` 之后才被创建，`read()` 仍会返回 `Err("文件不存在")`。需要时重新构造一个 `File`，或直接调用 `write`（它会刷新 `exist`）。

### 6. 借用检查摩擦

`mv` / `create` 接收 `&mut self` 却返回 `&Self`，返回的引用会延长可变借用，导致无法在同一个表达式里读取其它字段：

```rust
use nothings::filesystem::file::File;

let mut f = File::new("a.txt");

// 下面这行编译失败：cannot borrow `f.full_path` as immutable
// because it is also borrowed as mutable
// println!("{:?} {}", f.mv("b.txt").err, f.full_path);

let err = f.mv("b.txt").err.clone();   // 先克隆出错误，结束可变借用
println!("{err:?} {}", f.full_path);
```

### 7. 没有 `Debug` / `Clone`

`File` 与 `Dir` 都没有任何 derive，不能用 `{:?}` 打印，也不能克隆。调试时只能逐字段输出（字段都是 `pub`）。

## 备注

- `Dir` 的 `sub_files` / `sub_dirs` 是 [`AnySlice`](../any_slices/README.md)，因此 `list_files()` 上可以直接用 `has` / `filter` / `pluck` / `to_string` 等方法。注意 `AnySlice::remove_by_indexes` 与 `shuffle` 本身是失效的
- `File::write` 与 `Dir::zip` / `unzip` 都是**全量覆盖**，没有追加模式
- 所有路径参数都是 `&str`，不接受 `Path` / `PathBuf`，需要自己 `to_str().unwrap()`
