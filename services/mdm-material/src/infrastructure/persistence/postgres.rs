//! PostgreSQL repository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::Repository;

pub struct PostgresRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    // TODO: Implement repository methods
}
