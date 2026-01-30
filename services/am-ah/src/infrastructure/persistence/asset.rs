//! PostgreSQL implementation of AssetRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::AssetRepository;

pub struct PostgresAssetRepository {
    pool: Arc<PgPool>,
}

impl PostgresAssetRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for PostgresAssetRepository {
    // TODO: Implement repository methods
}
