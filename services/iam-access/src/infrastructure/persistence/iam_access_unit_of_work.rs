//! PostgreSQL Unit of Work 实现

use async_trait::async_trait;
use errors::{AppError, AppResult};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::error_mapper::map_sqlx_error;
use super::tx_repositories::{
    SharedTx, TxPermissionRepository, TxPolicyRepository, TxRolePermissionRepository,
    TxRoleRepository, TxUserRoleRepository,
};
use crate::domain::policy::PolicyRepository;
use crate::domain::role::{
    PermissionRepository, RolePermissionRepository, RoleRepository, UserRoleRepository,
};
use crate::domain::unit_of_work::{UnitOfWork, UnitOfWorkFactory};
use crate::infrastructure::persistence::outbox_repository::{
    OutboxEvent, OutboxRepository, PostgresOutboxRepository,
};

/// Postgres Unit of Work 工厂
pub struct PostgresUnitOfWorkFactory {
    pool: PgPool,
}

impl PostgresUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PostgresUnitOfWorkFactory {
    async fn begin(&self) -> AppResult<Box<dyn UnitOfWork>> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        Ok(Box::new(PostgresUnitOfWork::new(tx, self.pool.clone())))
    }
}

/// Postgres Unit of Work 实现
pub struct PostgresUnitOfWork {
    tx: SharedTx,
    // pool: PgPool, // Removed unused field
    role_repo: TxRoleRepository,
    permission_repo: TxPermissionRepository,
    role_permission_repo: TxRolePermissionRepository,
    user_role_repo: TxUserRoleRepository,
    policy_repo: TxPolicyRepository,
    outbox_repo: TxOutboxRepository,
}

impl PostgresUnitOfWork {
    pub fn new(tx: Transaction<'static, Postgres>, _pool: PgPool) -> Self {
        let tx = Arc::new(Mutex::new(Some(tx)));

        Self {
            tx: tx.clone(),
            // pool: pool.clone(), // Removed unused field
            role_repo: TxRoleRepository::new(tx.clone()),
            permission_repo: TxPermissionRepository::new(tx.clone()),
            role_permission_repo: TxRolePermissionRepository::new(tx.clone()),
            user_role_repo: TxUserRoleRepository::new(tx.clone()),
            policy_repo: TxPolicyRepository::new(tx.clone()),
            outbox_repo: TxOutboxRepository::new(tx.clone(), _pool),
        }
    }
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    fn roles(&self) -> &dyn RoleRepository {
        &self.role_repo
    }

    fn permissions(&self) -> &dyn PermissionRepository {
        &self.permission_repo
    }

    fn role_permissions(&self) -> &dyn RolePermissionRepository {
        &self.role_permission_repo
    }

    fn user_roles(&self) -> &dyn UserRoleRepository {
        &self.user_role_repo
    }

    fn policies(&self) -> &dyn PolicyRepository {
        &self.policy_repo
    }

    fn outbox(&self) -> &dyn OutboxRepository {
        &self.outbox_repo
    }

    async fn save_outbox_event(
        &self,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload_json: &str,
    ) -> AppResult<Uuid> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .as_mut()
            .ok_or_else(|| AppError::internal("Transaction consumed"))?;

        self.outbox_repo
            .insert_in_tx(tx, aggregate_type, aggregate_id, event_type, payload_json)
            .await
    }

    async fn commit(self: Box<Self>) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .take()
            .ok_or_else(|| AppError::internal("Transaction already consumed"))?;

        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn rollback(self: Box<Self>) -> AppResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard
            .take()
            .ok_or_else(|| AppError::internal("Transaction already consumed"))?;

        tx.rollback()
            .await
            .map_err(|e| AppError::database(format!("Failed to rollback transaction: {}", e)))?;

        Ok(())
    }
}

/// 事务感知的 Outbox Repository
pub struct TxOutboxRepository {
    // tx: SharedTx, // Removed unused field
    pool: PgPool,
}

impl TxOutboxRepository {
    pub fn new(_tx: SharedTx, pool: PgPool) -> Self {
        Self { /* tx, */ pool, }
    }
}

#[async_trait]
impl OutboxRepository for TxOutboxRepository {
    async fn insert_in_tx<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload_json: &str,
    ) -> AppResult<Uuid> {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(serde_json::from_str::<serde_json::Value>(payload_json).unwrap_or_default())
        .bind(chrono::Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(id)
    }

    async fn get_pending(&self, limit: i64) -> AppResult<Vec<OutboxEvent>> {
        PostgresOutboxRepository::new(self.pool.clone())
            .get_pending(limit)
            .await
    }

    async fn mark_published(&self, id: Uuid) -> AppResult<()> {
        PostgresOutboxRepository::new(self.pool.clone())
            .mark_published(id)
            .await
    }

    async fn mark_failed(&self, id: Uuid, error: &str) -> AppResult<()> {
        PostgresOutboxRepository::new(self.pool.clone())
            .mark_failed(id, error)
            .await
    }

    async fn delete_published(&self, before: chrono::DateTime<chrono::Utc>) -> AppResult<u64> {
        PostgresOutboxRepository::new(self.pool.clone())
            .delete_published(before)
            .await
    }
}
