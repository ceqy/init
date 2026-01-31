//! Command trait 定义

use async_trait::async_trait;
use errors::AppResult;

/// Command trait
pub trait Command: Send + Sync {
    type Result: Send;
}

/// Command Handler trait
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> AppResult<C::Result>;
}
