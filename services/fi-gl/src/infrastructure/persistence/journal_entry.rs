//! PostgreSQL implementation of JournalEntryRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::JournalEntryRepository;
use crate::error::ServiceResult;

pub struct PostgresJournalEntryRepository {
    pool: Arc<PgPool>,
}

impl PostgresJournalEntryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JournalEntryRepository for PostgresJournalEntryRepository {
    // TODO: Implement repository methods
}
