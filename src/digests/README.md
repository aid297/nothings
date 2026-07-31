# Digests — 哈希/摘要算法

`digests` 模块提供常用的哈希算法实现，包括 Bcrypt、MD5 和 SHA256。

## 模块一览

| 模块 | 结构体 | 说明 |
|------|--------|------|
| `bcrypt` | `Bcrypt` | 密码哈希与验证，基于 bcrypt 2b 算法 |
| `md5` | `Md5Hash` | MD5 哈希计算，适用于数据完整性校验 |
| `sha` | `Sha256Hash` | SHA256 哈希计算，适用于安全场景 |

---

## Bcrypt — 密码哈希与验证

基于 bcrypt 算法的密码哈希工具，默认使用 2b 版本，cost 控制计算复杂度（范围 4-31，默认 12）。

### 快速开始

```rust
use nothings::digests::bcrypt::Bcrypt;

let plaintext = "my_password";

// 使用默认 cost（12）生成哈希
let hashed = Bcrypt::new(plaintext).hash().unwrap();
println!("{}", hashed); // $2b$12$...

// 验证密码
let is_valid = Bcrypt::new(plaintext).check(&hashed).unwrap();
assert!(is_valid);

// 自定义 cost
let mut bcrypt = Bcrypt::new(plaintext);
bcrypt.set_cost(10);
let hashed = bcrypt.hash().unwrap();
```

### API 一览

| 方法 | 说明 |
|------|------|
| `Bcrypt::new(origin)` | 创建实例，传入明文密码 |
| `set_origin(origin)` | 重新设置明文密码 |
| `set_cost(cost)` | 设置计算复杂度（4-31，默认 12） |
| `hash()` | 生成哈希值，返回 `Result<String, BcryptError>` |
| `check(hashed)` | 验证密码是否匹配，返回 `Result<bool, BcryptError>` |

### 注意事项

- **仅用于密码存储**，不适用于通用数据哈希
- cost 值越高越安全，但计算越慢，建议根据性能需求调整
- 每次哈希结果不同（自动加盐），验证时需使用 `check` 方法

---

## MD5 — 数据完整性校验

基于 MD5 算法的哈希计算工具，输出 32 位十六进制字符串。

### 快速开始

```rust
use nothings::digests::md5::Md5Hash;

let data = "hello world";
let hashed = Md5Hash::new(data).hash();
println!("{}", hashed); // 5eb63bbbe01eeed093cb22bb8f5acdc3
```

### API 一览

| 方法 | 说明 |
|------|------|
| `Md5Hash::new(origin)` | 创建实例，传入原始数据 |
| `set_origin(origin)` | 重新设置原始数据 |
| `hash()` | 计算 MD5 哈希值，返回十六进制字符串 |

### 注意事项

- **不适用于密码等安全敏感场景**（MD5 已被证明存在碰撞攻击）
- 适用于文件校验、数据完整性验证等非安全场景

---

## SHA256 — 安全哈希

基于 SHA256 算法的哈希计算工具，输出 64 位十六进制字符串。

### 快速开始

```rust
use nothings::digests::sha::Sha256Hash;

let data = "hello world";
let hashed = Sha256Hash::new(data).hash();
println!("{}", hashed); // b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
```

### API 一览

| 方法 | 说明 |
|------|------|
| `Sha256Hash::new(origin)` | 创建实例，传入原始数据 |
| `set_origin(origin)` | 重新设置原始数据 |
| `hash()` | 计算 SHA256 哈希值，返回十六进制字符串 |

### 算法选择建议

| 场景 | 推荐算法 |
|------|----------|
| 密码存储与验证 | Bcrypt |
| 数据完整性校验（非安全） | MD5 |
| 数据完整性校验（安全） | SHA256 |
| 数字签名 / 证书 | SHA256 |
