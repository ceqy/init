//! PostgreSQL implementation of ServiceOrderRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::ServiceOrderRepository;
use crate::error::ServiceResult;

pub struct PostgresServiceOrderRepository {
    pool: Arc<PgPool>,
}

impl PostgresServiceOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServiceOrderRepository for PostgresServiceOrderRepository {
    // TODO: Implement repository methods
}
