# Time Info

时间字符串解析与常量定义。将 `"1h30m"`、`"2w3d"` 等时间字符串解析为 `std::time::Duration`。

## 时间常量

| 常量 | 值 |
|------|------|
| `SECOND` | 1 秒 |
| `MINUTE` | 60 秒 |
| `HOUR` | 3600 秒 |
| `DAY` | 86400 秒 |
| `WEEK` | 604800 秒 |

```rust
use nothings::texts::timer::*;
use std::time::Duration;

let d = Duration::from_secs(3600);
assert_eq!(d, HOUR);
```

## `parse_duration`

解析时间字符串，返回 `Result<Duration, String>`。

### 支持的单位

| 单位 | 含义 |
|------|------|
| `s` | 秒 |
| `m` | 分钟 |
| `h` | 小时 |
| `d` | 天 |
| `w` | 周 |

### 基本用法

```rust
use nothings::texts::timer::parse_duration;
use std::time::Duration;

assert_eq!(parse_duration("10s"), Ok(Duration::from_secs(10)));
assert_eq!(parse_duration("5m"), Ok(Duration::from_secs(300)));
assert_eq!(parse_duration("2h"), Ok(Duration::from_secs(7200)));
assert_eq!(parse_duration("1d"), Ok(Duration::from_secs(86400)));
assert_eq!(parse_duration("1w"), Ok(Duration::from_secs(604800)));
```

### 组合格式

多个单位可以连续书写，解析结果为各单位之和：

```rust
assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
assert_eq!(parse_duration("1d12h"), Ok(Duration::from_secs(129600)));
assert_eq!(parse_duration("2w3d"), Ok(Duration::from_secs(1468800)));
assert_eq!(parse_duration("1d2h3m4s"), Ok(Duration::from_secs(93784)));
```

### 无单位

末尾数字不带单位时，按前一个单位的下一级处理：

| 前一个单位 | 下一级单位 |
|------------|------------|
| `w` | 天 (d) |
| `d` | 小时 (h) |
| `h` | 分钟 (m) |
| `m` | 秒 (s) |
| `s` | 秒 (s) |

纯数字（无前一个单位）按秒处理：

```rust
// 纯数字按秒
assert_eq!(parse_duration("100"), Ok(Duration::from_secs(100)));

// 按前一个单位的下一级处理
assert_eq!(parse_duration("1h100"), Ok(Duration::from_secs(9600)));  // 1h + 100m
assert_eq!(parse_duration("1w2"), Ok(Duration::from_secs(777600)));  // 1w + 2d
assert_eq!(parse_duration("1d12"), Ok(Duration::from_secs(129600))); // 1d + 12h
assert_eq!(parse_duration("5m30"), Ok(Duration::from_secs(330)));    // 5m + 30s
```

### 错误处理

```rust
// 空字符串
assert!(parse_duration("").is_err());

// 未知单位
assert!(parse_duration("10x").is_err());

// 格式错误（单位在前数字在后）
assert!(parse_duration("s10").is_err());
```

## `describe_duration`

解析时间字符串，返回中文可读描述：

```rust
use nothings::texts::timer::describe_duration;

assert_eq!(describe_duration("1h30m"), Ok("1小时30分钟".to_string()));
assert_eq!(describe_duration("2d"), Ok("2天".to_string()));
assert_eq!(describe_duration("1w2d"), Ok("1周2天".to_string()));
assert_eq!(describe_duration("100"), Ok("100秒".to_string()));

// 无单位时按前一个单位的下一级
assert_eq!(describe_duration("1h100"), Ok("1小时100分钟".to_string()));
assert_eq!(describe_duration("1w2"), Ok("1周2天".to_string()));
assert_eq!(describe_duration("1d12"), Ok("1天12小时".to_string()));
```

## 与 `std::time::Duration` 互操作

`parse_duration` 返回标准库 `Duration`，可直接使用其所有方法：

```rust
let d = parse_duration("1h30m").unwrap();

d.as_secs();       // 5400
d.as_secs_f64();   // 5400.0
d.as_millis();     // 5400000

// 可直接传给需要 Duration 的 API
std::thread::sleep(d);
```

## 模块结构

```
text_time/
├── mod.rs          # 模块定义
├── text_info.rs    # 常量定义、parse_duration、describe_duration
└── README.md       # 本文档
```
