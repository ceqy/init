//! 备份码仓储接口

use async_trait::async_trait;
use cuba_common::{TenantId, UserId};
use cuba_errors::AppResult;

use crate::auth::domain::entities::{BackupCode, BackupCodeId};

#[async_trait]
pub trait BackupCodeRepository: Send + Sync {
    /// 保存备份码（自动使用备份码的 tenant_id）
    async fn save(&self, backup_code: &BackupCode) -> AppResult<()>;

    /// 批量保存备份码（自动使用备份码的 tenant_id）
    async fn save_batch(&self, backup_codes: &[BackupCode]) -> AppResult<()>;

    /// 根据 ID 查找备份码（带租户隔离）
    async fn find_by_id(&self, id: &BackupCodeId, tenant_id: &TenantId) -> AppResult<Option<BackupCode>>;

    /// 查找用户的所有可用备份码（带租户隔离）
    async fn find_available_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<Vec<BackupCode>>;

    /// 更新备份码（验证 tenant_id 匹配）
    async fn update(&self, backup_code: &BackupCode) -> AppResult<()>;

    /// 删除用户的所有备份码（带租户隔离）
    async fn delete_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<()>;

    /// 统计用户的可用备份码数量（带租户隔离）
    async fn count_available_by_user_id(&self, user_id: &UserId, tenant_id: &TenantId) -> AppResult<i64>;
}
