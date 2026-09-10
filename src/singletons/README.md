# Singletons — 单例宏

`singletons` 模块提供 `impl_singleton!` 宏，为任意类型一键生成「全局唯一实例 + 带锁访问」的三件套接口。

## ⚠️ 当前状态：外部 crate 无法使用

宏展开时会引用 `$crate::singletons::app::Singleton`，而这个 trait 被声明为 `pub(crate)`：

```rust
pub(crate) trait Singleton: Sized {
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}
```

结果是在 `nothings` 之外的任何 crate 里调用 `impl_singleton!` 都会**编译失败**，实测错误：

```text
error[E0603]: trait `Singleton` is private
 --> examples/probe.rs:9:1
  |
9 | impl_singleton!(Counter);
  | ^^^^^^^^^^^^^^^^^^^^^^^^^ private trait
  |
note: the trait `Singleton` is defined here
 --> src/singletons/app.rs:1:1
  |
1 | pub(crate) trait Singleton: Sized {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

注意报错发生在**宏展开处**，所以整个宏都失效——不只是 `with()` 用不了，连 `instance()` 和 `default()` 也不会被生成。

模块内部的单元测试（`src/singletons/test.rs`）能通过，是因为它们和 trait 在同一个 crate 里。

### 修复方式

把 `src/singletons/app.rs:1` 的可见性放宽一行即可：

```rust
pub trait Singleton: Sized {   // 原为 pub(crate)
```

在修复发布之前，请用下面「替代写法」小节的手写版本。

## 宏的用法（修复后 / crate 内部）

<!-- doccheck:skip —— 依赖 pub(crate) 的 Singleton trait，当前版本在外部 crate 编译失败，此处展示的是修复后的形态 -->

```rust
use nothings::impl_singleton;
use nothings::singletons::app::Singleton;   // with() 来自这个 trait，必须导入

struct Counter {
    value: u32,
}

impl Counter {
    // 必需：宏会调用 <Counter>::default_new() 来初始化实例
    fn default_new() -> Self {
        Counter { value: 0 }
    }

    fn increment(&mut self) -> &mut Self {
        self.value += 1;
        self
    }

    fn get(&self) -> u32 {
        self.value
    }
}

impl_singleton!(Counter);

// with() 拿到 &mut Counter，闭包的返回值会被透传出来
let n = Counter::with(|c| {
    c.increment();
    c.get()
});

// instance() 直接拿到 &'static Mutex<Counter>
Counter::instance().lock().unwrap().increment();

// default() 生成一个全新的、与单例无关的实例
let fresh = Counter::default();
```

### 前置要求

| 要求 | 说明 |
|------|------|
| 类型需有 `fn default_new() -> Self` | 宏内部调用 `<$t>::default_new()`，**可见性任意**（在宏调用处能看到即可），但名字必须是 `default_new` |
| 类型需 `Sized` | `Singleton: Sized` 约束 |
| 每个类型只能调用一次 | 重复调用会产生重复的 `instance` / `default` 定义，编译报重复定义 |

### 生成的接口

| 接口 | 签名 | 说明 |
|------|------|------|
| `instance()` | `-> &'static std::sync::Mutex<T>` | 首次调用时用 `default_new()` 惰性初始化，之后返回同一个实例 |
| `default()` | `-> Self` | **不是**单例，是每次新建一个 `default_new()` 的结果 |
| `with(f)` | `impl FnOnce(&mut T) -> R` `-> R` | 加锁后把 `&mut T` 交给闭包，返回闭包的结果 |

底层实现是 `OnceLock<Mutex<T>>`：

```rust
pub fn instance() -> &'static std::sync::Mutex<$t> {
    static INSTANCE: std::sync::OnceLock<std::sync::Mutex<$t>> = std::sync::OnceLock::new();
    INSTANCE.get_or_init(|| std::sync::Mutex::new(<$t>::default_new()))
}
```

`static` 声明在生成的函数体内，因此**每个类型有各自独立的一份**，不会互相干扰。

## 注意事项

### `default()` 与 `Default` trait 的关系

宏生成的是**固有方法** `pub fn default()`，不是 `impl Default`。后果：

- `Counter::default()` 调用的是宏生成的版本
- 若类型同时 `#[derive(Default)]` 或手写了 `impl Default`，固有方法**优先**，`Default::default()` 会被遮蔽
- 泛型代码里写 `T::default()`（依赖 `T: Default`）走的仍是 trait 版本，两者可能返回不同结果
- clippy 会对这个命名报 `should_implement_trait`

### 锁中毒会 panic

`with()` 内部是 `.lock().unwrap()`。如果某次 `with()` 的闭包里 panic 了，`Mutex` 会进入中毒状态，**之后所有 `with()` / `instance().lock()` 调用都会 panic**。单例是全局的，一次 panic 会永久性地废掉它。

需要容错就直接用 `instance()` 自己处理（接上例的 `Counter`）：

<!-- doccheck:skip —— 片段依赖上文定义的 Counter，非独立可编译示例 -->

```rust
match Counter::instance().lock() {
    Ok(mut c) => c.increment(),
    Err(poisoned) => poisoned.into_inner().increment(),   // 忽略中毒继续用
};
```

### 测试之间会互相污染

单例是进程级全局状态，`cargo test` 默认多线程并行跑，多个测试改同一个单例会互相影响。仓库自带的测试里就有这条注释：

```text
// 注意：由于单例是全局共享的，测试之间会互相影响
// 所以这里只验证 with 能正常访问
```

要么用 `cargo test -- --test-threads=1`，要么在断言里避开对初始值的依赖。

## 替代写法

在宏修好之前，直接手写展开结果即可，语义完全一致（下面这段已实测可编译运行）：

```rust
use std::sync::{Mutex, OnceLock};

struct Config {
    port: u16,
}

impl Config {
    fn default_new() -> Self {
        Config { port: 8080 }
    }

    fn get(&self) -> u16 {
        self.port
    }

    fn instance() -> &'static Mutex<Config> {
        static INSTANCE: OnceLock<Mutex<Config>> = OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(Config::default_new()))
    }

    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Config) -> R,
    {
        let mut ins = Self::instance().lock().unwrap();
        f(&mut ins)
    }

    fn default() -> Self {
        Config::default_new()
    }
}

Config::with(|c| c.port = 9090);
assert_eq!(Config::with(|c| c.get()), 9090);
assert_eq!(Config::default().port, 8080);
```

如果不需要「带锁的可变全局状态」，只是想要一个惰性常量，用 `OnceLock` 就够了，不必套 `Mutex`：

```rust
use std::sync::OnceLock;

fn config() -> &'static String {
    static C: OnceLock<String> = OnceLock::new();
    C.get_or_init(|| std::env::var("APP_NAME").unwrap_or_else(|_| "nothings".into()))
}

assert!(!config().is_empty());
```

## 备注

- `Singleton` trait 只有一个方法 `with`，且是**关联函数**（没有 `self` 参数），调用形式是 `T::with(|t| ...)` 而不是 `t.with(...)`
- `src/singletons/mod.rs` 只有 `pub mod app;` 一行，模块本身没有其它内容
- 该宏定义在 `src/singletons/app.rs`，但因为是 `#[macro_export]`，导出位置在 **crate 根**：`nothings::impl_singleton!`
