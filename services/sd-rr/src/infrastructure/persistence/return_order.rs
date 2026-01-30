//! PostgreSQL implementation of ReturnOrderRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::ReturnOrderRepository;
use crate::error::ServiceResult;

pub struct PostgresReturnOrderRepository {
    pool: Arc<PgPool>,
}

impl PostgresReturnOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReturnOrderRepository for PostgresReturnOrderRepository {
    // TODO: Implement repository methods
}
