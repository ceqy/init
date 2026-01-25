//! 备份码服务
//!
//! 提供备份码生成、验证和管理功能

use rand::Rng;
use sha2::{Digest, Sha256};

/// 备份码服务
pub struct BackupCodeService;

impl BackupCodeService {
    /// 生成备份码（10 个）
    pub fn generate_codes() -> Vec<String> {
        let mut rng = rand::thread_rng();
        (0..10)
            .map(|_| {
                // 生成 8 位数字备份码
                format!("{:08}", rng.r#gen_range(0..100_000_000))
            })
            .collect()
    }

    /// 哈希备份码
    pub fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证备份码
    pub fn verify_code(code: &str, hash: &str) -> bool {
        Self::hash_code(code) == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_codes() {
        let codes = BackupCodeService::generate_codes();
        
        // 应该生成 10 个备份码
        assert_eq!(codes.len(), 10);
        
        // 每个备份码应该是 8 位数字
        for code in codes {
            assert_eq!(code.len(), 8);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_hash_code() {
        let code = "12345678";
        let hash = BackupCodeService::hash_code(code);
        
        // SHA256 哈希应该是 64 个十六进制字符
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_code() {
        let code = "12345678";
        let hash = BackupCodeService::hash_code(code);
        
        // 正确的码应该验证成功
        assert!(BackupCodeService::verify_code(code, &hash));
        
        // 错误的码应该验证失败
        assert!(!BackupCodeService::verify_code("87654321", &hash));
    }
}
