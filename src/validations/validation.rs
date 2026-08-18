use crate::validations::error::ValidationError;
use crate::validations::field::Field;
use crate::validations::validator_struct_parser::NothingsValidatorStructParser;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// 列表值分隔符（用于 `in==`、`in!=`、`ex:` 中的值分隔）
pub enum SliceSplitChar {
    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `|`
    Pipe,
}

impl SliceSplitChar {
    pub fn as_str(&self) -> &str {
        match self {
            SliceSplitChar::Comma => ",",
            SliceSplitChar::Semicolon => ";",
            SliceSplitChar::Pipe => "|",
        }
    }
}

impl Default for SliceSplitChar {
    fn default() -> Self {
        SliceSplitChar::Comma
    }
}

/// 错误信息分隔符
pub enum ErrorSplitChar {
    /// `<br />`
    Web,
    /// `\n`
    Console,
}

impl ErrorSplitChar {
    pub fn as_str(&self) -> &str {
        match self {
            ErrorSplitChar::Web => "<br />",
            ErrorSplitChar::Console => "\n",
        }
    }
}

impl Default for ErrorSplitChar {
    fn default() -> Self {
        ErrorSplitChar::Console
    }
}

/// 全局分隔符配置
struct SplitConfig {
    slice_split_char: SliceSplitChar,
    error_split_char: ErrorSplitChar,
}

fn split_config() -> &'static Mutex<SplitConfig> {
    static CONFIG: OnceLock<Mutex<SplitConfig>> = OnceLock::new();
    CONFIG.get_or_init(|| {
        Mutex::new(SplitConfig {
            slice_split_char: SliceSplitChar::default(),
            error_split_char: ErrorSplitChar::default(),
        })
    })
}

/// 全局 ex 校验函数注册表
///
/// 函数签名：`Fn(field_name: &str, value: &str, kind: &str) -> Option<ValidationError>`
/// - `field_name`: 字段名
/// - `value`: 字段值的字符串表示
/// - `kind`: 字段类型
/// - 返回 `Some(ValidationError)` 表示校验失败
type ExCheckFn = Box<dyn Fn(&str, &str, &str) -> Option<ValidationError> + Send + Sync>;

fn ex_registry() -> &'static Mutex<HashMap<String, ExCheckFn>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, ExCheckFn>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 全局校验配置
///
/// 管理全局的分隔符设置和 ex 校验函数注册表。
/// 分隔符设置为全局生效，设置一次后所有 `Check` 实例自动使用。
pub struct Validation;

impl Validation {
    /// 设置全局值列表分隔符（用于 `in==`、`in!=`、`ex:` 中的值分隔）
    pub fn set_slice_split_char(c: SliceSplitChar) {
        split_config().lock().unwrap().slice_split_char = c;
    }

    /// 获取当前全局值列表分隔符
    pub fn slice_split_char() -> SliceSplitChar {
        match &split_config().lock().unwrap().slice_split_char {
            SliceSplitChar::Comma => SliceSplitChar::Comma,
            SliceSplitChar::Semicolon => SliceSplitChar::Semicolon,
            SliceSplitChar::Pipe => SliceSplitChar::Pipe,
        }
    }

    /// 设置全局错误信息分隔符
    pub fn set_error_split_char(c: ErrorSplitChar) {
        split_config().lock().unwrap().error_split_char = c;
    }

    /// 获取当前全局错误信息分隔符
    pub fn error_split_char() -> ErrorSplitChar {
        match &split_config().lock().unwrap().error_split_char {
            ErrorSplitChar::Web => ErrorSplitChar::Web,
            ErrorSplitChar::Console => ErrorSplitChar::Console,
        }
    }
    
    /// 注册一个全局 ex 校验函数
    ///
    /// ```ignore
    /// Validation::register_ex_check_fn("email-format", |field, value, kind| {
    ///     if !value.contains('@') {
    ///         Some(ValidationError { field: field.into(), message: "邮箱格式错误".into() })
    ///     } else {
    ///         None
    ///     }
    /// });
    /// ```
    pub fn register_ex_check_fn<F>(key: &str, check: F)
    where
        F: Fn(&str, &str, &str) -> Option<ValidationError> + Send + Sync + 'static,
    {
        let mut map = ex_registry().lock().unwrap();
        map.insert(key.to_string(), Box::new(check));
    }

    /// 调用已注册的全局 ex 校验函数
    pub fn call_ex_check_fn(
        field_name: &str,
        value: &str,
        kind: &str,
        fn_name: &str,
    ) -> Option<ValidationError> {
        let map = ex_registry().lock().unwrap();
        match map.get(fn_name) {
            Some(f) => f(field_name, value, kind),
            None => None,
        }
    }

    /// 检查指定的 ex 校验函数是否已注册
    pub fn has_ex_check_fn(key: &str) -> bool {
        let map = ex_registry().lock().unwrap();
        map.contains_key(key)
    }

    /// 校验入口：解析结构体字段
    ///
    /// 接收实现了 `#[derive(Nothings)]` 的结构体，解析所有字段的验证规则
    /// 返回解析后的字段列表，暂不执行规则校验
    pub fn validate<T: NothingsValidatorStructParser>(data: &T) -> Vec<Field> {
        data.parse_fields()
    }
}
