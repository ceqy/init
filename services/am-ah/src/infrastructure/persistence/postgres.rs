//! PostgreSQL repository implementation

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::Repository;

pub struct PostgresRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    // TODO: Implement repository methods
}
