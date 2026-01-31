//! Command/Query Bus

use async_trait::async_trait;
use errors::AppResult;

use crate::{Command, Query};

/// Command Bus trait
#[async_trait]
pub trait CommandBus: Send + Sync {
    async fn dispatch<C: Command + 'static>(&self, command: C) -> AppResult<C::Result>;
}

/// Query Bus trait
#[async_trait]
pub trait QueryBus: Send + Sync {
    async fn dispatch<Q: Query + 'static>(&self, query: Q) -> AppResult<Q::Result>;
}
