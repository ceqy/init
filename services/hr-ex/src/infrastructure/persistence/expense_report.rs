//! PostgreSQL implementation of ExpenseReportRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::ExpenseReportRepository;
use crate::error::ServiceResult;

pub struct PostgresExpenseReportRepository {
    pool: Arc<PgPool>,
}

impl PostgresExpenseReportRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExpenseReportRepository for PostgresExpenseReportRepository {
    // TODO: Implement repository methods
}
