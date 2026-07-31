use sha2::{Digest, Sha256};

/// SHA256 哈希工具
///
/// 提供基于 SHA256 算法的哈希计算功能。
/// SHA256 是一种安全的哈希算法，适用于数据完整性校验和密码哈希。
pub struct Sha256Hash<'a> {
    origin: &'a str,
}

impl<'a> Sha256Hash<'a> {
    pub fn new(origin: &'a str) -> Self {
        Sha256Hash { origin }
    }

    pub fn set_origin(&mut self, origin: &'a str) -> &Self {
        self.origin = origin;
        self
    }

    /// 计算 SHA256 哈希值，返回十六进制字符串
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.origin.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
