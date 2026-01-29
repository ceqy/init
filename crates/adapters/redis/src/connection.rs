//! Redis 连接管理

use cuba_errors::{AppError, AppResult};
use redis::Client;
use redis::aio::ConnectionManager;

/// 创建 Redis 连接管理器
pub async fn create_connection_manager(url: &str) -> AppResult<ConnectionManager> {
    let client = Client::open(url)
        .map_err(|e| AppError::internal(format!("Failed to create Redis client: {}", e)))?;

    ConnectionManager::new(client).await.map_err(|e| {
        AppError::internal(format!("Failed to create Redis connection manager: {}", e))
    })
}

/// 检查 Redis 连接
pub async fn check_connection(conn: &mut ConnectionManager) -> AppResult<()> {
    redis::cmd("PING")
        .query_async::<String>(conn)
        .await
        .map_err(|e| AppError::internal(format!("Redis health check failed: {}", e)))?;
    Ok(())
}
