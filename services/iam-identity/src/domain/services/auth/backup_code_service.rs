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

