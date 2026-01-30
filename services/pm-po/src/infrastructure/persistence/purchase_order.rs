//! PostgreSQL implementation of PurchaseOrderRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::PurchaseOrderRepository;
use crate::error::ServiceResult;

pub struct PostgresPurchaseOrderRepository {
    pool: Arc<PgPool>,
}

impl PostgresPurchaseOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseOrderRepository for PostgresPurchaseOrderRepository {
    // TODO: Implement repository methods
}
