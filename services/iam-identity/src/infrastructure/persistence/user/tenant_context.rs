//! 租户上下文辅助工具

use cuba_common::TenantId;
use cuba_errors::AppResult;
use sqlx::{PgConnection, PgPool};

/// 设置当前租户上下文（用于 RLS）
#[allow(dead_code)]
pub async fn set_tenant_context(conn: &mut PgConnection, tenant_id: &TenantId) -> AppResult<()> {
    sqlx::query(&format!(
        "SET LOCAL app.current_tenant_id = '{}'",
        tenant_id.0
    ))
    .execute(conn)
    .await
    .map_err(|e| cuba_errors::AppError::database(e.to_string()))?;

    Ok(())
}

/// 在事务中执行带租户上下文的操作
#[allow(dead_code)]
pub async fn with_tenant_context<F, T>(pool: &PgPool, tenant_id: &TenantId, f: F) -> AppResult<T>
where
    F: FnOnce(&mut PgConnection) -> futures::future::BoxFuture<'_, AppResult<T>>,
{
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| cuba_errors::AppError::database(e.to_string()))?;

    // 设置租户上下文
    set_tenant_context(&mut tx, tenant_id).await?;

    // 执行操作
    let result = f(&mut tx).await?;

    tx.commit()
        .await
        .map_err(|e| cuba_errors::AppError::database(e.to_string()))?;

    Ok(result)
}
