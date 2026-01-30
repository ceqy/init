//! PostgreSQL implementation of ValuationRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::ValuationRepository;

pub struct PostgresValuationRepository {
    pool: Arc<PgPool>,
}

impl PostgresValuationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ValuationRepository for PostgresValuationRepository {
    // TODO: Implement repository methods
}
