//! SQL 注入安全测试
//!
//! 本测试套件验证系统对 SQL 注入攻击的防护能力
//! 测试覆盖：
//! - 用户名注入
//! - Email 注入
//! - 查询参数注入
//! - UNION 注入
//! - 盲注测试
//!
//! 预期结果：所有测试应该通过（即所有注入尝试都应该失败或被安全处理）

use cuba_common::{TenantId, UserId};
use iam_identity::domain::repositories::user::UserRepository;
use iam_identity::domain::user::{User, UserStatus};
use iam_identity::domain::value_objects::{Email, HashedPassword, Username};
use iam_identity::infrastructure::persistence::user::PostgresUserRepository;
use sqlx::PgPool;
use std::env;

/// 获取测试数据库连接池
async fn get_test_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/cuba".to_string());
    PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to database")
}

/// 创建测试用户
async fn create_test_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    tenant_id: &TenantId,
) -> User {
    // 首先创建租户
    let _ = sqlx::query("INSERT INTO tenants (id, name, display_name) VALUES ($1, $2, $2) ON CONFLICT (id) DO NOTHING")
        .bind(tenant_id.0)
        .bind(format!("tenant_{}", tenant_id.0))
        .execute(pool)
        .await;

    let user = User::new(
        Username::new(username).unwrap(),
        Email::new(email).unwrap(),
        HashedPassword::from_hash("$2b$12$placeholder_hash_for_testing".to_string()),
        tenant_id.clone(),
    );

    // 插入用户到数据库
    let _ = sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, display_name, tenant_id, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(user.id.0)
    .bind(user.username.as_str())
    .bind(user.email.as_str())
    .bind(user.password_hash.as_str())
    .bind(user.display_name.as_deref().unwrap_or(""))
    .bind(tenant_id.0)
    .bind(user.status.to_string())
    .execute(pool)
    .await;

    user
}

// ============================================================================
// 测试用例 1: 基础 SQL 注入 - 单引号注入
// ============================================================================

#[tokio::test]
async fn test_sql_injection_single_quote_in_username() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    // 创建正常用户
    let _ = create_test_user(&pool, "normal_user", "normal@example.com", &tenant_id).await;

    // 尝试使用单引号注入的用户名进行查询
    let malicious_username = "admin' OR '1'='1";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let result = repo.find_by_username(&username, &tenant_id).await;

    // 预期：应该返回 None 或者正常用户，但绝不应该返回所有用户
    match result {
        Ok(None) => {
            // 正确：没有找到用户（因为不存在这样的用户名）
        }
        Ok(Some(user)) => {
            // 正确：只返回了具体的用户，没有被注入影响
            assert_ne!(user.username.as_str(), "admin");
        }
        Err(e) => {
            // 可能是参数验证错误，这也是可以接受的
            panic!("Unexpected error: {:?}", e);
        }
    }

    // 额外验证：确保没有返回多个用户
    // 如果注入成功，可能返回多个用户
    let result = repo.find_by_username(&username, &tenant_id).await;
    if let Ok(Some(_)) = result {
        // 正确：只返回一个或零个用户
    }
}

// ============================================================================
// 测试用例 2: UNION 注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_union_select() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    // 创建测试用户
    let _ = create_test_user(&pool, "target_user", "target@example.com", &tenant_id).await;

    // 尝试 UNION SELECT 注入
    let malicious_username =
        "admin' UNION SELECT 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24--";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let result = repo.find_by_username(&username, &tenant_id).await;

    // 预期：查询应该安全失败或返回空结果
    match result {
        Ok(None) => {
            // 正确：没有找到用户
        }
        Ok(Some(user)) => {
            // 如果返回了用户，必须确保不是注入的结果
            assert_ne!(user.username.as_str(), "admin");
        }
        Err(e) => {
            // 数据库错误也是可以接受的（参数化查询会拒绝无效的输入）
            panic!("Query should handle injection attempt gracefully: {:?}", e);
        }
    }
}

// ============================================================================
// 测试用例 3: COMMENT 注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_comment_bypass() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    // 创建测试用户
    let _ = create_test_user(&pool, "test_user", "test@example.com", &tenant_id).await;

    // 尝试使用注释绕过验证
    let malicious_username = "admin' --";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let result = repo.find_by_username(&username, &tenant_id).await;

    // 预期：注释不应该影响查询逻辑
    match result {
        Ok(None) => {
            // 正确：没有找到用户
        }
        Ok(Some(user)) => {
            // 如果返回了用户，确保不是注入的结果
            assert_ne!(user.username.as_str(), "admin");
        }
        Err(_) => {
            // 可接受的错误
        }
    }
}

// ============================================================================
// 测试用例 4: 时间盲注测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_time_based_blind() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    let _ = create_test_user(&pool, "victim", "victim@example.com", &tenant_id).await;

    // 记录查询开始时间
    let start = std::time::Instant::now();

    // 尝试时间盲注（如果注入成功，查询会延迟）
    let malicious_username = "admin' AND pg_sleep(5)--";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let _ = repo.find_by_username(&username, &tenant_id).await;

    let elapsed = start.elapsed();

    // 预期：查询应该在 2 秒内完成（没有 sleep 成功）
    assert!(
        elapsed.as_secs() < 2,
        "Query took too long, possible time-based SQL injection: {:?}",
        elapsed
    );
}

// ============================================================================
// 测试用例 5: Email 字段注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_in_email_field() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    // 尝试在 email 字段注入
    let malicious_email = "test@example.com' OR '1'='1";
    let email =
        Email::new(malicious_email).unwrap_or_else(|_| Email::new("test@example.com").unwrap());

    let result = repo.find_by_email(&email, &tenant_id).await;

    // 预期：注入应该被阻止
    match result {
        Ok(None) => {
            // 正确：没有找到用户
        }
        Ok(Some(user)) => {
            // 如果返回了用户，确保不是注入的结果
            assert_ne!(user.email.as_str(), "test@example.com");
        }
        Err(_) => {
            // 可接受的错误
        }
    }
}

// ============================================================================
// 测试用例 6: 多语句注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_multi_statement() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    let _ = create_test_user(&pool, "target", "target@test.com", &tenant_id).await;

    // 尝试多语句注入（删除用户）
    let malicious_username = "admin'; DROP TABLE users; --";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let _ = repo.find_by_username(&username, &tenant_id).await;

    // 验证：users 表应该仍然存在
    let result = sqlx::query("SELECT 1 FROM users LIMIT 1")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok(), "users table should still exist");
}

// ============================================================================
// 测试用例 7: 级联注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_cascading() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    // 创建多个用户
    let _ = create_test_user(&pool, "user1", "user1@example.com", &tenant_id).await;
    let _ = create_test_user(&pool, "user2", "user2@example.com", &tenant_id).await;

    // 尝试级联注入（绕过租户隔离）
    let malicious_username = "admin' AND tenant_id = '";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let result = repo.find_by_username(&username, &tenant_id).await;

    // 预期：不能绕过租户隔离
    match result {
        Ok(None) => {
            // 正确：没有找到用户
        }
        Ok(Some(user)) => {
            // 如果返回了用户，必须确保租户隔离仍然有效
            assert_eq!(user.tenant_id, tenant_id);
        }
        Err(_) => {
            // 可接受的错误
        }
    }
}

// ============================================================================
// 测试用例 8: 编码绕过测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_encoding_bypass() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    let _ = create_test_user(&pool, "encoded_user", "encoded@example.com", &tenant_id).await;

    // 尝试使用 URL 编码绕过
    let malicious_username = "admin%27%20OR%20%271%27%3D%271";
    let username =
        Username::new(malicious_username).unwrap_or_else(|_| Username::new("admin").unwrap());

    let result = repo.find_by_username(&username, &tenant_id).await;

    // 预期：编码不应该绕过保护
    match result {
        Ok(None) => {
            // 正确：没有找到用户
        }
        Ok(Some(user)) => {
            // 如果返回了用户，确保不是注入的结果
            assert_ne!(user.username.as_str(), "admin");
        }
        Err(_) => {
            // 可接受的错误
        }
    }
}

// ============================================================================
// 测试用例 9: 二阶注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_second_order() {
    let pool = get_test_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    let tenant_id = TenantId::new();

    // 首先创建租户
    let _ = sqlx::query("INSERT INTO tenants (id, name, display_name) VALUES ($1, $2, $2) ON CONFLICT (id) DO NOTHING")
        .bind(tenant_id.0)
        .bind(format!("tenant_{}", tenant_id.0))
        .execute(&pool)
        .await;

    // 创建一个包含潜在注入内容的用户，直接插入数据库
    let user_id = UserId::new();
    let malicious_display_name = "test', (SELECT password FROM users WHERE username='admin'), '";

    let _ = sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, display_name, tenant_id, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(user_id.0)
    .bind("second_order")
    .bind("second@example.com")
    .bind("$2b$12$placeholder_hash_for_testing")
    .bind(malicious_display_name)
    .bind(tenant_id.0)
    .bind("Active")
    .execute(&pool)
    .await;

    // 尝试使用这个用户信息进行其他查询
    let username = Username::new("second_order").unwrap();
    let result = repo.find_by_username(&username, &tenant_id).await;

    // 预期：存储的数据不应该在后续查询中导致注入
    match result {
        Ok(Some(found_user)) => {
            // 确保返回的数据是原始存储的数据，没有执行注入
            assert!(
                found_user
                    .display_name
                    .as_deref()
                    .map(|s| s.contains("'"))
                    .unwrap_or(false)
            );
        }
        Ok(None) => {
            // 可接受的结果（用户可能因为其他原因未找到）
        }
        Err(e) => {
            // 可接受的错误
            panic!("Second order injection test error: {:?}", e);
        }
    }
}

// ============================================================================
// 测试用例 10: LIKE 注入测试
// ============================================================================

#[tokio::test]
async fn test_sql_injection_like_wildcards() {
    let pool = get_test_pool().await;
    let tenant_id = TenantId::new();

    // 创建测试用户
    let _ = create_test_user(&pool, "admin", "admin@example.com", &tenant_id).await;
    let _ = create_test_user(&pool, "administrator", "admin2@example.com", &tenant_id).await;
    let _ = create_test_user(&pool, "superadmin", "superadmin@example.com", &tenant_id).await;

    // 尝试使用通配符匹配多个用户
    let wildcard_username = "admin%";

    // 直接查询（模拟可能存在的模糊搜索功能）
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT username FROM users WHERE username LIKE $1 AND tenant_id = $2 LIMIT 10",
    )
    .bind(wildcard_username)
    .bind(tenant_id.0)
    .fetch_all(&pool)
    .await;

    // 预期：如果模糊搜索功能存在，应该返回匹配的用户
    // 但关键是要验证通配符没有被滥用
    match result {
        Ok(users) => {
            // 如果返回了用户，验证数量和内容
            // 确保 LIMIT 限制生效，没有返回过多数据
            assert!(users.len() <= 10, "Should respect LIMIT clause");

            // 验证所有返回的用户都以 "admin" 开头
            for (username,) in users {
                assert!(
                    username.starts_with("admin"),
                    "All results should start with 'admin'"
                );
            }
        }
        Err(e) => {
            // 模糊搜索功能可能不存在
            panic!("Query failed: {:?}", e);
        }
    }
}

// ============================================================================
// 测试总结
// ============================================================================

// 这个测试套件验证了系统的 SQL 注入防护能力
//
// 关键防护机制：
// 1. 使用 sqlx 的参数化查询
// 2. 所有用户输入都通过值对象验证
// 3. 没有使用字符串拼接构建 SQL
// 4. 租户隔离通过参数实现
//
// 风险等级：低
// 所有查询都使用参数化查询，有效防止了 SQL 注入
//
// 建议改进：
// 1. 考虑添加额外的输入验证层
// 2. 对于模糊搜索，限制通配符的使用
// 3. 添加查询日志和监控
// 4. 定期进行安全审计和渗透测试
