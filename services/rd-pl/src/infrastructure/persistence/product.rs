//! PostgreSQL implementation of ProductRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::ProductRepository;

pub struct PostgresProductRepository {
    pool: Arc<PgPool>,
}

impl PostgresProductRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    // TODO: Implement repository methods
}
