//! PostgreSQL implementation of VendorInvoiceRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::VendorInvoiceRepository;
use crate::error::ServiceResult;

pub struct PostgresVendorInvoiceRepository {
    pool: Arc<PgPool>,
}

impl PostgresVendorInvoiceRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VendorInvoiceRepository for PostgresVendorInvoiceRepository {
    // TODO: Implement repository methods
}
