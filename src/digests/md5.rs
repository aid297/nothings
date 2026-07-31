use md5::{Digest, Md5};

/// MD5 哈希工具
///
/// 提供基于 MD5 算法的哈希计算和验证功能。
/// 注意：MD5 不适用于密码等安全敏感场景，建议用于数据完整性校验。
pub struct Md5Hash<'a> {
    origin: &'a str,
}

impl<'a> Md5Hash<'a> {
    pub fn new(origin: &'a str) -> Self {
        Md5Hash { origin }
    }

    pub fn set_origin(&mut self, origin: &'a str) -> &Self {
        self.origin = origin;
        self
    }

    /// 计算 MD5 哈希值，返回十六进制字符串
    pub fn hash(&self) -> String {
        let mut hasher = Md5::new();
        hasher.update(self.origin.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
