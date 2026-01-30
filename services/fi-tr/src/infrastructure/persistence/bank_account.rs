//! PostgreSQL implementation of BankAccountRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::BankAccountRepository;
use crate::error::ServiceResult;

pub struct PostgresBankAccountRepository {
    pool: Arc<PgPool>,
}

impl PostgresBankAccountRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BankAccountRepository for PostgresBankAccountRepository {
    // TODO: Implement repository methods
}
