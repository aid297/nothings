use nothings_macros::Nothings;
use crate::validations::error::ValidationError;
use crate::validations::validation::Validation;
use crate::validations::check::Check;
use crate::validations::checker::Checker;

#[test]
fn test_register_and_call_ex_fn() {
    // 注册一个全局 ex 校验函数
    Validation::register_ex_check_fn("check-not-empty", |field, value, _kind| {
        if value.is_empty() {
            Some(ValidationError {
                field: field.into(),
                message: "不能为空".into(),
            })
        } else {
            None
        }
    });

    // 调用校验函数 - 空值应失败
    let result = Validation::call_ex_check_fn("用户名", "", "string", "check-not-empty");
    assert!(result.is_some(), "空值应校验失败");
    assert_eq!(result.unwrap().message, "不能为空");

    // 调用校验函数 - 非空值应通过
    let result = Validation::call_ex_check_fn("用户名", "alice", "string", "check-not-empty");
    assert!(result.is_none(), "非空值应校验通过");
}

#[test]
fn test_ex_fn_global() {
    // 注册一个全局函数
    Validation::register_ex_check_fn("global-check", |_field, value, _kind| {
        if value.len() > 10 {
            Some(ValidationError {
                field: "test".into(),
                message: "长度不能超过10".into(),
            })
        } else {
            None
        }
    });

    // 在不同地方调用，都能找到这个函数
    assert!(Validation::has_ex_check_fn("global-check"));
    
    let result = Validation::call_ex_check_fn("test", "short", "string", "global-check");
    assert!(result.is_none());
    
    let result = Validation::call_ex_check_fn("test", "this is a very long string", "string", "global-check");
    assert!(result.is_some());
}

#[test]
fn test_ex_fn_not_found() {
    // 调用未注册的函数应返回 None
    let result = Validation::call_ex_check_fn("field", "value", "string", "non-existent-fn");
    assert!(result.is_none(), "未注册的函数应返回 None");
}

#[test]
fn test_has_ex_check_fn() {
    Validation::register_ex_check_fn("exists-fn", |_, _, _| None);
    
    assert!(Validation::has_ex_check_fn("exists-fn"));
    assert!(!Validation::has_ex_check_fn("not-exists-fn"));
}

#[test]
fn test_ex_fn_with_kind() {
    // 注册一个根据 kind 区分的校验函数
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

    // usize 类型校验
    let result = Validation::call_ex_check_fn("age", "150", "usize", "check-range");
    assert!(result.is_some());
    
    let result = Validation::call_ex_check_fn("age", "50", "usize", "check-range");
    assert!(result.is_none());

    // string 类型校验
    let result = Validation::call_ex_check_fn("name", "a".repeat(60).as_str(), "string", "check-range");
    assert!(result.is_some());
    
    let result = Validation::call_ex_check_fn("name", "short", "string", "check-range");
    assert!(result.is_none());
}

#[derive(Nothings)]
struct A<'a >{
    #[validator[rule="(min>3)(max<=4)" name="姓名" kind="string"]]
    name: String,
    #[validator[rule="(min>20)(max<=100)" name="年龄" kind="usize"]]
    age: usize,
    #[validator[rule="(min>1)" name="性别"]]
    sex: &'a str,
    #[validator[rule="(max<999)" name="身高"]]
    height: f64,
}

#[test]
fn test_struct_parser(){
    let a = A{
        name: "alice".into(),
        age: 18,
        sex: "female",
        height: 170.5,
    };
    
    let fields = Validation::validate(&a);
    
    assert_eq!(fields.len(), 4, "应解析出 4 个字段");
    
    // 验证每个字段的解析结果
    assert_eq!(fields[0].name, "姓名");
    assert_eq!(fields[0].kind, "string");
    assert_eq!(fields[0].origin, "alice");
    assert_eq!(fields[0].rules, vec!["min>3", "max<=4"]);
    
    assert_eq!(fields[1].name, "年龄");
    assert_eq!(fields[1].kind, "usize");
    assert_eq!(fields[1].origin, "18");
    assert_eq!(fields[1].rules, vec!["min>20", "max<=100"]);
    
    assert_eq!(fields[2].name, "性别");
    assert_eq!(fields[2].origin, "female");
    assert_eq!(fields[2].rules, vec!["min>1"]);
    
    assert_eq!(fields[3].name, "身高");
    assert_eq!(fields[3].origin, "170.5");
    assert_eq!(fields[3].rules, vec!["max<999"]);
}

// 测试 Option 字段的 required 规则
#[derive(Nothings)]
struct FormWithOption {
    #[validator[rule="(required)" name="邮箱" kind="string"]]
    email: Option<String>,
    #[validator[rule="(min>3)" name="昵称" kind="string"]]
    nickname: Option<String>,
}

#[test]
fn test_option_required_some() {
    let form = FormWithOption {
        email: Some("test@example.com".into()),
        nickname: Some("alice".into()),
    };
    
    let fields = Validation::validate(&form);
    // email 有 required，Some 值应正常解析
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "邮箱");
    assert_eq!(fields[0].origin, "test@example.com");
    assert!(fields[0].is_option);
    assert_eq!(fields[0].rules, vec!["required"]);
    
    assert_eq!(fields[1].name, "昵称");
    assert_eq!(fields[1].origin, "alice");
    assert!(fields[1].is_option);
}

#[test]
fn test_option_required_none() {
    let form = FormWithOption {
        email: None,
        nickname: None,
    };
    
    let fields = Validation::validate(&form);
    // email 有 required，None 时仍应生成字段
    assert_eq!(fields.len(), 1, "只有 required 的 Option=None 应生成字段");
    assert_eq!(fields[0].name, "邮箱");
    assert_eq!(fields[0].origin, "");
    assert!(fields[0].is_option);
    assert_eq!(fields[0].rules, vec!["required"]);
    
    // nickname 没有 required，None 时应跳过
}

#[test]
fn test_option_required_check() {
    // email=None 且 required 应校验失败
    let form = FormWithOption {
        email: None,
        nickname: None,
    };
    
    let checker = Check::new(form);
    let err = checker.check();
    assert!(err.is_some(), "required 字段为 None 应校验失败");
    assert_eq!(err.unwrap().field, "邮箱");
}

#[test]
fn test_option_some_check_pass() {
    // email=Some 应校验通过
    let form = FormWithOption {
        email: Some("test@example.com".into()),
        nickname: Some("alice".into()),
    };
    
    let checker = Check::new(form);
    let err = checker.check();
    assert!(err.is_none(), "required 字段有值应校验通过");
}

// ========== min/max/size/in/not-in 规则测试 ==========

#[derive(Nothings)]
struct RuleTestForm {
    #[validator[rule="(min>18)" name="年龄" kind="usize"]]
    age: usize,
    #[validator[rule="(max<=100)" name="分数" kind="usize"]]
    score: usize,
    #[validator[rule="(size==5)" name="编码" kind="string"]]
    code: String,
    #[validator[rule="(in==admin,user,guest)" name="角色" kind="string"]]
    role: String,
    #[validator[rule="(in!=banned,blocked)" name="状态" kind="string"]]
    status: String,
}

#[test]
fn test_min_rule_pass() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABCDE".into(),
        role: "admin".into(),
        status: "active".into(),
    };
    let checker = Check::new(form);
    assert!(checker.check().is_none(), "所有规则应通过");
}

#[test]
fn test_min_rule_fail() {
    let form = RuleTestForm {
        age: 10, // min>18 应失败
        score: 80,
        code: "ABCDE".into(),
        role: "admin".into(),
        status: "active".into(),
    };
    let checker = Check::new(form);
    let err = checker.check().unwrap();
    assert_eq!(err.field, "年龄");
}

#[test]
fn test_max_rule_fail() {
    let form = RuleTestForm {
        age: 25,
        score: 150, // max<=100 应失败
        code: "ABCDE".into(),
        role: "admin".into(),
        status: "active".into(),
    };
    let checker = Check::new(form);
    let err = checker.check().unwrap();
    assert_eq!(err.field, "分数");
}

#[test]
fn test_size_rule_pass() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABCDE".into(), // 长度=5，size==5 应通过
        role: "admin".into(),
        status: "active".into(),
    };
    let checker = Check::new(form);
    assert!(checker.check().is_none());
}

#[test]
fn test_size_rule_fail() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABC".into(), // 长度=3，size==5 应失败
        role: "admin".into(),
        status: "active".into(),
    };
    let checker = Check::new(form);
    let err = checker.check().unwrap();
    assert_eq!(err.field, "编码");
}

#[test]
fn test_in_rule_pass() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABCDE".into(),
        role: "user".into(), // 在 admin,user,guest 中
        status: "active".into(),
    };
    let checker = Check::new(form);
    assert!(checker.check().is_none());
}

#[test]
fn test_in_rule_fail() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABCDE".into(),
        role: "superadmin".into(), // 不在 admin,user,guest 中
        status: "active".into(),
    };
    let checker = Check::new(form);
    let err = checker.check().unwrap();
    assert_eq!(err.field, "角色");
}

#[test]
fn test_not_in_rule_pass() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABCDE".into(),
        role: "admin".into(),
        status: "active".into(), // 不在 banned,blocked 中，应通过
    };
    let checker = Check::new(form);
    assert!(checker.check().is_none());
}

#[test]
fn test_not_in_rule_fail() {
    let form = RuleTestForm {
        age: 25,
        score: 80,
        code: "ABCDE".into(),
        role: "admin".into(),
        status: "banned".into(), // 在 banned,blocked 中，应失败
    };
    let checker = Check::new(form);
    let err = checker.check().unwrap();
    assert_eq!(err.field, "状态");
}

// 多规则组合测试
#[derive(Nothings)]
struct MultiRuleForm {
    #[validator[rule="(min>0)(max<=150)" name="年龄" kind="usize"]]
    age: usize,
}

#[test]
fn test_multi_rules_all_pass() {
    let form = MultiRuleForm { age: 25 };
    let checker = Check::new(form);
    assert!(checker.check().is_none());
}

#[test]
fn test_multi_rules_first_fail() {
    let form = MultiRuleForm { age: 0 }; // min>0 失败
    let checker = Check::new(form);
    assert!(checker.check().is_some());
}

#[test]
fn test_multi_rules_second_fail() {
    let form = MultiRuleForm { age: 200 }; // max<=150 失败
    let checker = Check::new(form);
    assert!(checker.check().is_some());
}