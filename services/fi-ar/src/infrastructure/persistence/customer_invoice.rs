//! PostgreSQL implementation of CustomerInvoiceRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::CustomerInvoiceRepository;
use crate::error::ServiceResult;

pub struct PostgresCustomerInvoiceRepository {
    pool: Arc<PgPool>,
}

impl PostgresCustomerInvoiceRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerInvoiceRepository for PostgresCustomerInvoiceRepository {
    // TODO: Implement repository methods
}
