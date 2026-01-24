//! Query trait 定义

use async_trait::async_trait;
use cuba_errors::AppResult;

/// Query trait
pub trait Query: Send + Sync {
    type Result: Send;
}

/// Query Handler trait
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> AppResult<Q::Result>;
}
