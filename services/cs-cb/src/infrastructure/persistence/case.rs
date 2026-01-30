//! PostgreSQL implementation of CaseRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::CaseRepository;
use crate::error::ServiceResult;

pub struct PostgresCaseRepository {
    pool: Arc<PgPool>,
}

impl PostgresCaseRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CaseRepository for PostgresCaseRepository {
    // TODO: Implement repository methods
}
