use std::fmt;

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "field '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}