//! PostgreSQL implementation of PriceListRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::PriceListRepository;
use crate::error::ServiceResult;

pub struct PostgresPriceListRepository {
    pool: Arc<PgPool>,
}

impl PostgresPriceListRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PriceListRepository for PostgresPriceListRepository {
    // TODO: Implement repository methods
}
