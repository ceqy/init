//! PostgreSQL implementation of MeterRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::MeterRepository;

pub struct PostgresMeterRepository {
    pool: Arc<PgPool>,
}

impl PostgresMeterRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MeterRepository for PostgresMeterRepository {
    // TODO: Implement repository methods
}
