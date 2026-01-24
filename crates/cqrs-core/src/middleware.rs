//! Middleware 定义

use async_trait::async_trait;
use cuba_errors::AppResult;

/// Command Middleware trait
#[async_trait]
pub trait CommandMiddleware: Send + Sync {
    async fn before<C: Send + Sync>(&self, command: &C) -> AppResult<()>;
    async fn after<C: Send + Sync, R: Send + Sync>(&self, command: &C, result: &AppResult<R>);
}

/// Query Middleware trait
#[async_trait]
pub trait QueryMiddleware: Send + Sync {
    async fn before<Q: Send + Sync>(&self, query: &Q) -> AppResult<()>;
    async fn after<Q: Send + Sync, R: Send + Sync>(&self, query: &Q, result: &AppResult<R>);
}

/// 日志中间件
pub struct LoggingMiddleware;

#[async_trait]
impl CommandMiddleware for LoggingMiddleware {
    async fn before<C: Send + Sync>(&self, _command: &C) -> AppResult<()> {
        tracing::debug!("Executing command");
        Ok(())
    }

    async fn after<C: Send + Sync, R: Send + Sync>(&self, _command: &C, result: &AppResult<R>) {
        match result {
            Ok(_) => tracing::debug!("Command executed successfully"),
            Err(e) => tracing::error!("Command failed: {}", e),
        }
    }
}

#[async_trait]
impl QueryMiddleware for LoggingMiddleware {
    async fn before<Q: Send + Sync>(&self, _query: &Q) -> AppResult<()> {
        tracing::debug!("Executing query");
        Ok(())
    }

    async fn after<Q: Send + Sync, R: Send + Sync>(&self, _query: &Q, result: &AppResult<R>) {
        match result {
            Ok(_) => tracing::debug!("Query executed successfully"),
            Err(e) => tracing::error!("Query failed: {}", e),
        }
    }
}
