use bcrypt::{BcryptError, DEFAULT_COST, hash, verify};

/// Bcrypt 密码哈希工具
///
/// 提供基于 bcrypt 算法的密码哈希和验证功能。
/// 默认使用 2b 版本，cost 参数控制计算复杂度（范围 4-31，默认 12）。
pub struct Bcrypt<'a> {
    origin: &'a str,
    cost: u32,
}

impl<'a> Bcrypt<'a> {
    pub fn new(origin: &'a str) -> Self {
        Bcrypt {
            origin,
            cost: DEFAULT_COST,
        }
    }

    pub fn set_origin(&mut self, origin: &'a str) -> &Self {
        self.origin = origin;
        self
    }

    pub fn set_cost(&mut self, cost: u32) -> &Self {
        self.cost = cost;
        self
    }

    pub fn hash(&self) -> Result<String, BcryptError> {
        hash(&self.origin, self.cost)
    }

    pub fn check(&self, hashed: &String) -> Result<bool, BcryptError> {
        verify(&self.origin, hashed)
    }
}
