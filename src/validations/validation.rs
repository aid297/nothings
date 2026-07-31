use crate::validations::error::ValidationError;
use crate::validations::field::Field;
use crate::validations::aid_rs_validator_struct_parser::NothingsValidatorStructParser;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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

/// 全局 ex 校验函数管理
pub struct Validation;

impl Validation {
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
    pub fn call_ex_check_fn(field_name: &str, value: &str, kind: &str, fn_name: &str) -> Option<ValidationError> {
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
