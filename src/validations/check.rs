use crate::validations::{*};
use crate::validations::checker::Checker;
use crate::validations::aid_rs_validator_struct_parser::NothingsValidatorStructParser;

pub struct Check<T> {
    data: T,
}

/// 从规则字符串中提取操作符和值
/// 输入格式：`operator+value`（如 `>10`、`<=100`、`=20`、`!=a,b,c`）
/// 返回：(操作符, 值) 如 (`>`, `10`)
fn parse_op_value(s: &str) -> Option<(&str, &str)> {
    if let Some(v) = s.strip_prefix(">=") {
        Some((">=", v))
    } else if let Some(v) = s.strip_prefix("<=") {
        Some(("<=", v))
    } else if let Some(v) = s.strip_prefix("!=") {
        Some(("!=", v))
    } else if let Some(v) = s.strip_prefix(">") {
        Some((">", v))
    } else if let Some(v) = s.strip_prefix("<") {
        Some(("<", v))
    } else if let Some(v) = s.strip_prefix("=") {
        Some(("=", v))
    } else {
        None
    }
}

/// 数值比较：将 origin 和 target 都解析为 f64 进行比较
fn check_numeric(origin: &str, op: &str, target: &str, field_name: &str) -> Option<error::ValidationError> {
    let origin_val = if let Ok(v) = origin.parse::<f64>(){
        v
    }else{
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

fn check_by_op(origin: &f64, target: &f64, op:&str, field_name: &str) -> Option<error::ValidationError>{
    let passed = match op {
        ">" => origin > target,
        "<" => origin < target,
        ">=" => origin >= target,
        "<=" => origin <= target,
        "=" => (origin - target).abs() < f64::EPSILON,
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
fn check_size(origin: &str, op: &str, target: &str, field_name: &str) -> Option<error::ValidationError> {
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

/// 列表检查：判断 origin 是否在逗号分隔的列表中
fn check_in_list(origin: &str, op: &str, list_str: &str, field_name: &str) -> Option<error::ValidationError> {
    let list: Vec<&str> = list_str.split(',').map(|s| s.trim()).collect();
    let in_list = list.contains(&origin);
    
    let passed = match op {
        "=" => in_list,
        "!=" => !in_list,
        _ => true,
    };
    
    if !passed {
        let msg = if op == "=" {
            format!("字段 '{}' 的值 '{}' 不在允许列表 [{}] 中", field_name, origin, list_str)
        } else {
            format!("字段 '{}' 的值 '{}' 在不允许的列表 [{}] 中", field_name, origin, list_str)
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
        
        for field in fields {
            for rule in &field.rules {
                // required 规则：检查 Option 字段是否为 None
                if rule == "required" && field.is_option && field.origin.is_empty() {
                    return Some(error::ValidationError {
                        field: field.name.clone(),
                        message: format!("字段 '{}' 是必填项", field.name),
                    });
                }
                
                // ex 规则格式：`ex:`fn1,fn2,fn3``
                if let Some(fn_names) = rule.strip_prefix("ex:`") {
                    if let Some(fn_names) = fn_names.strip_suffix('`') {
                        for fn_name in fn_names.split(',') {
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
                    }
                }
                
                // 通用规则格式：`key:`operator+value``
                // 解析 key 和 backtick 包裹的内容
                if let Some(colon_pos) = rule.find(':') {
                    let key = &rule[..colon_pos];
                    let rest = &rule[colon_pos + 1..];
                    
                    // 去除 backtick 包裹
                    if let Some(inner) = rest.strip_prefix('`').and_then(|s| s.strip_suffix('`')) {
                        if let Some((op, value)) = parse_op_value(inner) {
                            match key {
                                "min" | "max" => {
                                    if let Some(err) = check_numeric(&field.origin, op, value, &field.name) {
                                        return Some(err);
                                    }
                                }
                                "size" => {
                                    if let Some(err) = check_size(&field.origin, op, value, &field.name) {
                                        return Some(err);
                                    }
                                }
                                "in" => {
                                    if let Some(err) = check_in_list(&field.origin, op, value, &field.name) {
                                        return Some(err);
                                    }
                                }
                                _ => {}
                            }
                        }
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
