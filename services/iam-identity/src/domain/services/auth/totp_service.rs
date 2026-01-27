//! TOTP 服务
//!
//! 提供 TOTP 生成、验证和 QR 码生成功能

use cuba_errors::{AppError, AppResult};
use data_encoding::BASE32;
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

/// TOTP 服务
pub struct TotpService {
    issuer: String,
}

impl TotpService {
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }

    /// 生成 TOTP secret
    pub fn generate_secret(&self) -> AppResult<String> {
        // 生成 20 字节随机数据
        let mut rng = rand::thread_rng();
        let secret_bytes: Vec<u8> = (0..20).map(|_| rng.r#gen()).collect();

        // Base32 编码
        Ok(BASE32.encode(&secret_bytes))
    }

    /// 生成 QR 码 URL（otpauth:// 格式）
    pub fn generate_qr_code_url(
        &self,
        username: &str,
        secret: &str,
    ) -> AppResult<String> {
        // 构建 otpauth:// URL
        let url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
            urlencoding::encode(&self.issuer),
            urlencoding::encode(username),
            secret,
            urlencoding::encode(&self.issuer)
        );
        Ok(url)
    }

    /// 验证 TOTP 码
    pub fn verify_code(
        &self,
        _username: &str,
        secret: &str,
        code: &str,
    ) -> AppResult<bool> {
        let totp = self.create_totp(secret)?;
        Ok(totp.check_current(code).unwrap_or(false))
    }

    /// 创建 TOTP 实例
    fn create_totp(&self, secret: &str) -> AppResult<TOTP> {
        let secret = Secret::Encoded(secret.to_string())
            .to_bytes()
            .map_err(|e| AppError::internal(format!("Invalid secret: {}", e)))?;

        TOTP::new(
            Algorithm::SHA1,
            6,  // 6 位数字
            1,  // 1 步时间窗口
            30, // 30 秒有效期
            secret,
        )
        .map_err(|e| AppError::internal(format!("Failed to create TOTP: {}", e)))
    }
}

