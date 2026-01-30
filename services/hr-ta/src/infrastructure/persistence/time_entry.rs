//! PostgreSQL implementation of TimeEntryRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::TimeEntryRepository;
use crate::error::ServiceResult;

pub struct PostgresTimeEntryRepository {
    pool: Arc<PgPool>,
}

impl PostgresTimeEntryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimeEntryRepository for PostgresTimeEntryRepository {
    // TODO: Implement repository methods
}
