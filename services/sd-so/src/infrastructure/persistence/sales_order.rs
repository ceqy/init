//! PostgreSQL implementation of SalesOrderRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::SalesOrderRepository;
use crate::error::ServiceResult;

pub struct PostgresSalesOrderRepository {
    pool: Arc<PgPool>,
}

impl PostgresSalesOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SalesOrderRepository for PostgresSalesOrderRepository {
    // TODO: Implement repository methods
}
