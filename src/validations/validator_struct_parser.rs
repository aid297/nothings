use crate::validations::field::Field;

/// 由派生宏 `#[derive(Nothings)]` 自动实现的 trait
/// 
/// 用于解析结构体字段上的 `#[validator(...)]` 属性
pub trait NothingsValidatorStructParser {
    /// 解析所有字段及其验证规则
    fn parse_fields(&self) -> Vec<Field>;
}
