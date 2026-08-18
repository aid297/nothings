# Validations

基于属性宏的声明式结构体字段校验模块。通过 `#[derive(Nothings)]` + `#[validator[...]]` 在结构体上声明校验规则，运行时由 `Check` 执行校验。

## 快速开始

```rust
use nothings::validations::check::Check;
use nothings::validations::checker::Checker;
use nothings::validations::Nothings;

#[derive(Nothings)]
struct UserForm {
    #[validator[rule="(required)(min>3)(max<=20)" name="用户名" kind="string"]]
    username: String,
    #[validator[rule="(min>18)(max<=100)" name="年龄" kind="usize"]]
    age: usize,
    #[validator[rule="(in==admin,user,guest)" name="角色" kind="string"]]
    role: String,
}

let form = UserForm {
    username: "alice".into(),
    age: 25,
    role: "admin".into(),
};

let checker = Check::new(form);
match checker.check() {
    None => println!("校验通过"),
    Some(err) => println!("校验失败: {} - {}", err.field, err.message),
}
```

## 属性语法

每个需要校验的字段使用 `#[validator[...]]` 声明，支持以下属性：

| 属性 | 必填 | 说明 |
|------|------|------|
| `rule` | 是 | 校验规则字符串 |
| `name` | 是 | 字段显示名（用于错误信息） |
| `kind` | 否 | 字段类型，影响 `min`/`max` 的比较方式。不指定时使用 Rust 字段类型 |
| `nested` | 否 | 标记嵌套结构体，递归解析子字段 |

## 规则语法

规则用 `(...)` 包裹，操作符直接跟在规则名后，多规则连续书写：

```
(rule1)(rule2)(rule3)
```

### 支持的操作符

| 操作符 | 含义 |
|--------|------|
| `>` | 大于 |
| `<` | 小于 |
| `>=` | 大于等于 |
| `<=` | 小于等于 |
| `==` | 等于 |
| `!=` | 不等于 |

### 内置规则

#### `required` / `!`

标记 `Option<T>` 字段为必填。当值为 `None` 时校验失败。`!` 是 `required` 的简写形式，两者完全等价。

```rust
// 以下两种写法等价
#[validator[rule="(required)" name="邮箱" kind="string"]]
email: Option<String>,

#[validator[rule="(!)" name="邮箱" kind="string"]]
email: Option<String>,
```

> `Option<T>` 字段未标记 `required/!` 时，若值为 `None` 则自动跳过该字段的所有校验。

#### `min` / `max`

数值或字符串长度比较：

- **数值类型**（`size`、`usize`、`f64` 等）：比较数值大小
- **字符串类型**（`String`、`&str`、`kind="string"`）：比较字符长度

```rust
// 数值：年龄必须 > 18 且 <= 100
#[validator[rule="(min>18)(max<=100)" name="年龄" kind="usize"]]
age: usize,

// 字符串：用户名长度必须 > 3 且 <= 20
#[validator[rule="(min>3)(max<=20)" name="用户名" kind="string"]]
username: String,
```

#### `size`

始终按字符串长度比较，支持所有操作符：

```rust
// 编码长度必须恰好为 5
#[validator[rule="(size==5)" name="编码" kind="string"]]
code: String,

// 备注长度不能为 4
#[validator[rule="(size!=4)" name="备注" kind="string"]]
remark: String,

// 标题长度必须大于 2
#[validator[rule="(size>2)" name="标题" kind="string"]]
title: String,
```

#### `in` / `in!=`

列表成员检查。值列表的分隔符由 `Validation::set_slice_split_char` 全局配置（默认 `,`），可选值：`,`（Comma）、`;`（Semicolon）、`|`（Pipe）。

设置方式：

```rust
Validation::set_slice_split_char(SliceSplitChar::Pipe); // 全局使用 | 分隔
```

示例：

```rust
// 角色必须在 admin,user,guest 中
#[validator[rule="(in==admin,user,guest)" name="角色" kind="string"]]
role: String,

// 状态不能在 banned,blocked 中
#[validator[rule="(in!=banned,blocked)" name="状态" kind="string"]]
status: String,
```

#### `ex` (自定义校验函数)

引用通过 `Validation::register_ex_check_fn` 注册的全局自定义校验函数。多个函数名用分隔符分隔：

```rust
#[validator[rule="(ex:email-format,not-disposable)" name="邮箱" kind="string"]]
email: String,
```

### 多规则组合

规则按顺序执行，遇到第一个失败即返回错误：

```rust
// 必填 + 长度范围 + 自定义校验
#[validator[rule="(required)(min>3)(max<=20)(ex:custom-check)" name="用户名" kind="string"]]
username: Option<String>,
```

## 自定义校验函数 (ex)

通过 `Validation::register_ex_check_fn` 注册全局校验函数，在规则中用 `ex:fn-name` 引用。

### 函数签名

```rust
Fn(field_name: &str, value: &str, kind: &str) -> Option<ValidationError>
```

| 参数 | 说明 |
|------|------|
| `field_name` | 字段显示名 |
| `value` | 字段值的字符串表示 |
| `kind` | 字段类型 |
| 返回值 | `Some(ValidationError)` 表示失败，`None` 表示通过 |

### 示例

```rust
use nothings::validations::validation::Validation;
use nothings::validations::error::ValidationError;

// 注册
Validation::register_ex_check_fn("email-format", |field, value, _kind| {
    if value.contains('@') {
        None
    } else {
        Some(ValidationError {
            field: field.into(),
            message: "邮箱格式错误".into(),
        })
    }
});

// 根据 kind 区分逻辑
Validation::register_ex_check_fn("check-range", |field, value, kind| {
    match kind {
        "usize" => {
            if let Ok(n) = value.parse::<usize>() {
                if n > 100 {
                    return Some(ValidationError {
                        field: field.into(),
                        message: "数值不能超过100".into(),
                    });
                }
            }
            None
        }
        "string" => {
            if value.len() > 50 {
                Some(ValidationError {
                    field: field.into(),
                    message: "字符串长度不能超过50".into(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
});

// 检查是否已注册
assert!(Validation::has_ex_check_fn("email-format"));
```

## 分隔符配置

`in==`、`in!=`、`ex:` 中的值列表分隔符通过 `Validation` 的静态方法全局配置。设置一次后，所有 `Check` 实例自动使用。

### SliceSplitChar（值列表分隔符）

| 变体 | 实际字符 | 说明 |
|------|----------|------|
| `Comma` (默认) | `,` | 逗号 |
| `Semicolon` | `;` | 分号 |
| `Pipe` | `\|` | 竖线 |

### ErrorSplitChar（错误信息分隔符）

| 变体 | 实际字符串 | 说明 |
|------|------------|------|
| `Console` (默认) | `\n` | 换行符 |
| `Web` | `<br />` | HTML 换行 |

### 使用示例

```rust
use nothings::validations::validation::{Validation, SliceSplitChar, ErrorSplitChar};

// 全局设置，一次配置后所有 Check 实例自动使用
Validation::set_slice_split_char(SliceSplitChar::Pipe);
Validation::set_error_split_char(ErrorSplitChar::Web);

// 之后直接创建 Check 即可，无需额外配置
let checker = Check::new(form);
let err = checker.check();
```

配置分隔符后，规则中的值列表需使用对应分隔符：

```rust
// 全局设置为 Pipe 后
Validation::set_slice_split_char(SliceSplitChar::Pipe);

// 值列表用 | 分隔
#[validator[rule="(in==admin|user|guest)" name="角色" kind="string"]]
role: String,

// ex 函数名也用 | 分隔
#[validator[rule="(ex:fn1|fn2|fn3)" name="邮箱" kind="string"]]
email: String,
```

## 嵌套结构体

使用 `nested` 标记嵌套结构体，递归解析子字段并自动添加前缀：

```rust
#[derive(Nothings)]
struct Address {
    #[validator[rule="(required)(min>1)" name="城市" kind="string"]]
    city: String,
}

#[derive(Nothings)]
struct UserForm {
    #[validator[rule="(required)" name="用户名" kind="string"]]
    username: String,
    #[validator[nested]]
    address: Address,
}
```

嵌套字段的前缀规则：
- 指定了 `name` 时，用 `name` 的值作为前缀
- 未指定 `name` 时，用 Rust 字段名作为前缀

```rust
// 指定 name="地址"，子字段错误名为 "地址.城市"
#[validator[nested name="地址"]]
address: Address,

// 未指定 name，子字段错误名为 "address.城市"
#[validator[nested]]
address: Address,
```

## Option 字段行为

| 字段值 | 有 `required` | 无 `required` |
|--------|---------------|---------------|
| `Some(v)` | 正常解析并校验 | 正常解析并校验 |
| `None` | 生成空字段，`check()` 返回错误 | 跳过该字段，不参与校验 |

## 模块结构

```
validations/
├── mod.rs                    # 模块定义，重新导出 Nothings 宏
├── validation.rs             # Validation 配置、ex 函数注册表、分隔符定义
├── check.rs                  # Check 校验执行器，规则解析与匹配
├── checker.rs                # Checker trait 定义
├── field.rs                  # Field 数据结构（解析后的字段信息）
├── error.rs                  # ValidationError 错误类型
├── validator_struct_parser.rs # NothingsValidatorStructParser trait（由宏自动实现）
└── test.rs                   # 测试用例
```

### 核心类型关系

```
#[derive(Nothings)]
        │
        ▼  自动生成
NothingsValidatorStructParser::parse_fields()
        │
        ▼  返回
Vec<Field>  ──────►  Check::check()  ──────►  Option<ValidationError>
                          │
                          │ 读取全局配置
                          ▼
                   Validation (静态方法)
                   ├── set_slice_split_char() / slice_split_char()
                   └── set_error_split_char() / error_split_char()
```
