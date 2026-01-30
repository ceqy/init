//! PostgreSQL implementation of ProjectRepository

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::ProjectRepository;

pub struct PostgresProjectRepository {
    pool: Arc<PgPool>,
}

impl PostgresProjectRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PostgresProjectRepository {
    // TODO: Implement repository methods
}
