//! 数据库错误映射工具
//! 
//! 提供统一的 SQLx 错误到 AppError 的转换

use cuba_errors::AppError;

/// 将 SQLx 错误转换为 AppError，区分不同错误类型
pub fn map_sqlx_error(e: sqlx::Error) -> AppError {
    match e {
        sqlx::Error::RowNotFound => {
            AppError::not_found("Record not found")
        }
        sqlx::Error::Database(db_err) => {
            if let Some(code) = db_err.code() {
                match code.as_ref() {
                    // PostgreSQL 约束违规代码
                    "23505" => AppError::conflict("Duplicate entry violates unique constraint"),
                    "23503" => AppError::validation("Foreign key constraint violation"),
                    "23514" => AppError::validation("Check constraint violation"),
                    "23502" => AppError::validation("Not null constraint violation"),
                    "22001" => AppError::validation("String data too long"),
                    "22P02" => AppError::validation("Invalid input syntax"),
                    _ => AppError::database(format!("Database error ({}): {}", code, db_err)),
                }
            } else {
                AppError::database(db_err.to_string())
            }
        }
        sqlx::Error::PoolTimedOut => {
            AppError::internal("Database connection pool timeout")
        }
        sqlx::Error::PoolClosed => {
            AppError::internal("Database connection pool is closed")
        }
        sqlx::Error::Protocol(msg) => {
            AppError::internal(format!("Database protocol error: {}", msg))
        }
        _ => AppError::database(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_not_found() {
        let err = map_sqlx_error(sqlx::Error::RowNotFound);
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn test_pool_timeout() {
        let err = map_sqlx_error(sqlx::Error::PoolTimedOut);
        assert!(matches!(err, AppError::Internal(_)));
    }
}
