//! PostgreSQL implementation of CostCenterRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::CostCenterRepository;
use crate::error::ServiceResult;

pub struct PostgresCostCenterRepository {
    pool: Arc<PgPool>,
}

impl PostgresCostCenterRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CostCenterRepository for PostgresCostCenterRepository {
    // TODO: Implement repository methods
}
