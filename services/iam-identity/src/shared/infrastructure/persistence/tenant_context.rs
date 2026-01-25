//! 租户上下文辅助工具

use cuba_common::TenantId;
use cuba_errors::AppResult;
use sqlx::{PgConnection, PgPool};

/// 设置当前租户上下文（用于 RLS）
pub async fn set_tenant_context(conn: &mut PgConnection, tenant_id: &TenantId) -> AppResult<()> {
    sqlx::query(&format!(
        "SET LOCAL app.current_tenant_id = '{}'",
        tenant_id.0
    ))
    .execute(conn)
    .await?;

    Ok(())
}

/// 在事务中执行带租户上下文的操作
pub async fn with_tenant_context<F, T>(
    pool: &PgPool,
    tenant_id: &TenantId,
    f: F,
) -> AppResult<T>
where
    F: FnOnce(&mut PgConnection) -> futures::future::BoxFuture<'_, AppResult<T>>,
{
    let mut tx = pool.begin().await?;

    // 设置租户上下文
    set_tenant_context(&mut *tx, tenant_id).await?;

    // 执行操作
    let result = f(&mut *tx).await?;

    tx.commit().await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_set_tenant_context(pool: PgPool) {
        let tenant_id = TenantId::new();
        let mut conn = pool.acquire().await.unwrap();

        let result = set_tenant_context(&mut *conn, &tenant_id).await;
        assert!(result.is_ok());

        // 验证上下文已设置
        let (value,): (String,) = sqlx::query_as(
            "SELECT current_setting('app.current_tenant_id', true)"
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap();

        assert_eq!(value, tenant_id.0.to_string());
    }
}
