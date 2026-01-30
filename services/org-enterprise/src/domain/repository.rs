//! Repository interfaces

use async_trait::async_trait;

#[async_trait]
pub trait Repository: Send + Sync {
    // TODO: Define repository methods
}
