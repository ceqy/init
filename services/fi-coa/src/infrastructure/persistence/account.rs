//! PostgreSQL implementation of AccountRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::AccountRepository;
use crate::error::ServiceResult;

pub struct PostgresAccountRepository {
    pool: Arc<PgPool>,
}

impl PostgresAccountRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    // TODO: Implement repository methods
}
