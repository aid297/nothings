use crate::validations::checker::Checker;
use crate::validations::validator_struct_parser::NothingsValidatorStructParser;
use crate::validations::*;

pub struct Check<T> {
    data: T,
}

/// 从规则字符串中提取规则名、操作符和值
/// 新格式：`min>10`、`max<=100`、`size==200`、`in!=a,b,c`
/// 返回：(规则名, 操作符, 值) 如 (`min`, `>`, `10`)
fn parse_rule_parts(rule: &str) -> Option<(&str, &str, &str)> {
    let bytes = rule.as_bytes();
    let len = bytes.len();

    for i in 0..len {
        let b = bytes[i];
        // 找到第一个操作符字符
        if b == b'>' || b == b'<' || b == b'=' || b == b'!' {
            let key = &rule[..i];
            if key.is_empty() {
                return None;
            }
            // 检查是否为双字符操作符
            if i + 1 < len {
                let next = bytes[i + 1];
                match (b, next) {
                    (b'>', b'=') => return Some((key, ">=", &rule[i + 2..])),
                    (b'<', b'=') => return Some((key, "<=", &rule[i + 2..])),
                    (b'!', b'=') => return Some((key, "!=", &rule[i + 2..])),
                    (b'=', b'=') => return Some((key, "==", &rule[i + 2..])),
                    _ => {}
                }
            }
            // 单字符操作符（`!` 单独出现无效）
            if b == b'!' {
                return None;
            }
            return Some((key, &rule[i..i + 1], &rule[i + 1..]));
        }
    }
    None
}

/// 数值比较：将 origin 和 target 都解析为 f64 进行比较
fn check_numeric(
    origin: &str,
    op: &str,
    target: &str,
    field_name: &str,
) -> Option<error::ValidationError> {
    let origin_val = if let Ok(v) = origin.parse::<f64>() {
        v
    } else {
        return Some(error::ValidationError {
            field: field_name.to_string(),
            message: format!("值错误，需要数字：{}", origin),
        });
    };
    let target_val = if let Ok(v) = target.parse::<f64>() {
        v
    } else {
        return Some(error::ValidationError {
            field: field_name.to_string(),
            message: format!("规则错误，需要数字：{}", target),
        });
    };

    check_by_op(&origin_val, &target_val, op, field_name)
}

fn check_by_op(
    origin: &f64,
    target: &f64,
    op: &str,
    field_name: &str,
) -> Option<error::ValidationError> {
    let passed = match op {
        ">" => origin > target,
        "<" => origin < target,
        ">=" => origin >= target,
        "<=" => origin <= target,
        "==" => (origin - target).abs() < f64::EPSILON,
        "!=" => (origin - target).abs() >= f64::EPSILON,
        _ => true,
    };

    if !passed {
        Some(error::ValidationError {
            field: field_name.to_string(),
            message: format!("字段 '{}' 不满足条件 {}{}", field_name, op, target),
        })
    } else {
        None
    }
}

/// 长度比较：取 origin 的字符长度进行比较
fn check_size(
    origin: &str,
    op: &str,
    target: &str,
    field_name: &str,
) -> Option<error::ValidationError> {
    let size = origin.len() as f64;
    let target_val = if let Ok(v) = target.parse::<f64>() {
        v
    } else {
        return Some(error::ValidationError {
            field: field_name.to_string(),
            message: format!("规则错误，需要数字：{}", target),
        });
    };

    check_by_op(&size, &target_val, op, field_name)
}

/// 列表检查：判断 origin 是否在分隔的列表中
fn check_in_list(
    origin: &str,
    op: &str,
    list_str: &str,
    field_name: &str,
    split_char: &str,
) -> Option<error::ValidationError> {
    let list: Vec<&str> = list_str.split(split_char).map(|s| s.trim()).collect();
    let in_list = list.contains(&origin);

    let passed = match op {
        "==" => in_list,
        "!=" => !in_list,
        _ => true,
    };

    if !passed {
        let msg = if op == "==" {
            format!(
                "字段 '{}' 的值 '{}' 不在允许列表 [{}] 中",
                field_name, origin, list_str
            )
        } else {
            format!(
                "字段 '{}' 的值 '{}' 在不允许的列表 [{}] 中",
                field_name, origin, list_str
            )
        };
        Some(error::ValidationError {
            field: field_name.to_string(),
            message: msg,
        })
    } else {
        None
    }
}

impl<T> Checker<T> for Check<T>
where
    T: NothingsValidatorStructParser,
{
    fn new(data: T) -> Self {
        Check { data }
    }

    fn check(&self) -> Option<error::ValidationError> {
        let fields = self.dispatch_struct();
        let split_char_config = validation::Validation::slice_split_char();
        let split_char = split_char_config.as_str();

        for field in fields {
            for rule in &field.rules {
                // required 规则：检查 Option 字段是否为 None
                if (rule == "required" || rule == "!") && field.is_option && field.origin.is_empty()
                {
                    return Some(error::ValidationError {
                        field: field.name.clone(),
                        message: format!("字段 '{}' 是必填项", field.name),
                    });
                }

                // ex 规则格式：`ex:fn1[,、|、;]fn2[, | ;]fn3[,、|、;]`
                if let Some(fn_names) = rule.strip_prefix("ex:") {
                    for fn_name in fn_names.split(split_char) {
                        let fn_name = fn_name.trim();
                        if let Some(err) = validation::Validation::call_ex_check_fn(
                            &field.name,
                            &field.origin,
                            &field.kind,
                            fn_name,
                        ) {
                            return Some(err);
                        }
                    }
                    continue;
                }

                // 通用规则新格式：`key op value`（如 `min>10`、`size==200`、`in==a[,、|、;]b[,、|、;]c`）
                if let Some((key, op, value)) = parse_rule_parts(rule) {
                    match key {
                        "min" | "max" => {
                            // 字符串类型用长度比较，其他类型用数值比较
                            if field.kind == "string"
                                || field.kind == "& str"
                                || field.kind == "String"
                            {
                                if let Some(err) = check_size(&field.origin, op, value, &field.name)
                                {
                                    return Some(err);
                                }
                            } else if let Some(err) =
                                check_numeric(&field.origin, op, value, &field.name)
                            {
                                return Some(err);
                            }
                        }
                        "size" => {
                            if let Some(err) = check_size(&field.origin, op, value, &field.name) {
                                return Some(err);
                            }
                        }
                        "in" => {
                            if let Some(err) =
                                check_in_list(&field.origin, op, value, &field.name, split_char)
                            {
                                return Some(err);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        None
    }

    fn dispatch_struct(&self) -> Vec<field::Field> {
        self.data.parse_fields()
    }
}


