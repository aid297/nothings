# Timer

时间字符串解析模块。将 `"1h30m"`、`"2w3d"` 等时间字符串解析为 `std::time::Duration`。

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

## Timer trait

```rust
pub trait Time {
    fn new() -> Time;
    fn with_text(s: &str) -> Result<Time, String>;
    fn get_duration(&self) -> Duration;
    fn to_chinese(&self) -> Result<String, String>;
}
```

| 方法 | 说明 |
|------|------|
| `Time::new()` | 创建空的 Timer 实例（0 秒） |
| `Time::with_text(s)` | 解析时间字符串，返回 `Result<Time, String>` |
| `get_duration()` | 获取解析后的 `Duration` |
| `to_chinese()` | 返回中文可读描述 |

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
use nothings::texts::prelude::Time;
use std::time::Duration;

let t = Time::with_text("10s").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(10));

let t = Time::with_text("5m").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(300));

let t = Time::with_text("2h").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(7200));

let t = Time::with_text("1d").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(86400));

let t = Time::with_text("1w").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(604800));
```

### 组合格式

多个单位可以连续书写，解析结果为各单位之和：

```rust
let t = Time::with_text("1h30m").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(5400));

let t = Time::with_text("1d12h").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(129600));

let t = Time::with_text("2w3d").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(1468800));
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
let t = Time::with_text("100").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(100));

// 按前一个单位的下一级处理
let t = Time::with_text("1h100").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(9600));  // 1h + 100m

let t = Time::with_text("1w2").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(777600)); // 1w + 2d

let t = Time::with_text("1d12").unwrap();
assert_eq!(t.get_duration(), Duration::from_secs(129600)); // 1d + 12h
```

### 错误处理

```rust
// 空字符串
assert!(Time::with_text("").is_err());

// 未知单位
assert!(Time::with_text("10x").is_err());

// 格式错误（单位在前数字在后）
assert!(Time::with_text("s10").is_err());
```

### 中文描述

返回中文可读描述：

```rust
use nothings::texts::prelude::Time;

let t = Time::with_text("1h30m").unwrap();
assert_eq!(t.to_chinese(), Ok("1小时30分钟".to_string()));

let t = Time::with_text("2d").unwrap();
assert_eq!(t.to_chinese(), Ok("2天".to_string()));

let t = Time::with_text("1w2d").unwrap();
assert_eq!(t.to_chinese(), Ok("1周2天".to_string()));

let t = Time::with_text("100").unwrap();
assert_eq!(t.to_chinese(), Ok("100秒".to_string()));

// 无单位时按前一个单位的下一级
let t = Time::with_text("1h100").unwrap();
assert_eq!(t.to_chinese(), Ok("1小时100分钟".to_string()));

let t = Time::with_text("1w2").unwrap();
assert_eq!(t.to_chinese(), Ok("1周2天".to_string()));
```

## 与 `std::time::Duration` 互操作

通过 `get_duration()` 获取标准库 `Duration`，可直接使用其所有方法：

```rust
use nothings::texts::prelude::Time;

let t = Time::with_text("1h30m").unwrap();
let d = t.get_duration();

d.as_secs();       // 5400
d.as_secs_f64();   // 5400.0
d.as_millis();     // 5400000

// 可直接传给需要 Duration 的 API
std::thread::sleep(d);
```

## 模块结构

```
texts/
├── mod.rs        # 模块定义
├── timer.rs      # Time trait、Time 结构体、时间常量
├── volumer.rs    # Volume trait、Volume 结构体
├── prelude.rs    # 统一导出 Time、Volumer
└── README.md     # 本文档
```
