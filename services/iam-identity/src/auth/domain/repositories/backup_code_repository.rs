//! 备份码仓储接口

use async_trait::async_trait;
use cuba_common::UserId;
use cuba_errors::AppResult;

use crate::auth::domain::entities::{BackupCode, BackupCodeId};

#[async_trait]
pub trait BackupCodeRepository: Send + Sync {
    /// 保存备份码
    async fn save(&self, backup_code: &BackupCode) -> AppResult<()>;

    /// 批量保存备份码
    async fn save_batch(&self, backup_codes: &[BackupCode]) -> AppResult<()>;

    /// 根据 ID 查找备份码
    async fn find_by_id(&self, id: &BackupCodeId) -> AppResult<Option<BackupCode>>;

    /// 查找用户的所有可用备份码
    async fn find_available_by_user_id(&self, user_id: &UserId) -> AppResult<Vec<BackupCode>>;

    /// 更新备份码
    async fn update(&self, backup_code: &BackupCode) -> AppResult<()>;

    /// 删除用户的所有备份码
    async fn delete_by_user_id(&self, user_id: &UserId) -> AppResult<()>;

    /// 统计用户的可用备份码数量
    async fn count_available_by_user_id(&self, user_id: &UserId) -> AppResult<i64>;
}
