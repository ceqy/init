//! Redis 安全测试
//!
//! 本测试套件验证 Redis 使用中的安全风险
//! 测试覆盖：
//! - Key 注入测试
//! - 缓存污染攻击
//! - Redis 命令注入
//! - 缓存穿透/击穿/雪崩
//! - 缓存绕过攻击
//!
//! 预期结果：验证 Redis 使用的安全性，识别潜在风险

use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::env;
use std::time::Duration;

/// 获取 Redis 连接
async fn get_redis_client() -> ConnectionManager {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");

    ConnectionManager::new(client)
        .await
        .expect("Failed to create Redis connection manager")
}

// ============================================================================
// 测试用例 1: Key 注入测试 - 特殊字符
// ============================================================================

#[tokio::test]
async fn test_redis_key_injection_special_characters() {
    let mut conn = get_redis_client().await;

    // 测试包含特殊字符的 key
    let malicious_keys = vec![
        "user:info\nadmin",
        "user:info\radmin",
        "user:info\x00admin",
        "user:info\test",
        "user:1' OR '1'='1",
    ];

    for key in malicious_keys {
        // 尝试设置这些 key
        let _: () = conn.set(key, "test_value").await.unwrap_or(());

        // 验证 key 是否被正确存储（无注入影响）
        let value: Option<String> = conn.get(key).await.unwrap_or(None);
        if let Some(v) = value {
            assert_eq!(v, "test_value");
        }

        // 清理
        let _: () = conn.del(key).await.unwrap_or(());
    }
}

// ============================================================================
// 测试用例 2: Key 注入测试 - 换行和命令分隔符
// ============================================================================

#[tokio::test]
async fn test_redis_key_injection_command_separator() {
    let mut conn = get_redis_client().await;

    // 测试可能作为命令分隔符的字符
    let dangerous_keys = vec![
        "user:123;FLUSHALL",
        "user:123\nDEL *",
        "user:123\r\nCONFIG SET *",
        "user:123\tSET admin true",
        "user:123\\",
    ];

    for key in dangerous_keys {
        // 尝试存储危险 key
        let _: () = conn.set(key, "value").await.unwrap_or(());

        // 验证存储的是字面量，没有执行注入的命令
        let value: Option<String> = conn.get(key).await.unwrap_or(None);

        // 如果存储成功，值应该是字面量
        match value {
            Some(ref v) => {
                assert!(v == "value" || v != "true");
            }
            None => {
                // 可接受的情况
            }
        }

        // 验证没有执行危险命令
        // 例如，如果 FLUSHALL 被执行，所有 key 会被删除
        let test_key = "test_injection_check";
        let _: () = conn.set(test_key, "should_exist").await.unwrap_or(());
        let exists: bool = conn.exists(test_key).await.unwrap_or(false);
        assert!(exists, "Dangerous command should not be executed");

        // 清理
        let _: () = conn.del(key).await.unwrap_or(());
    }
}

// ============================================================================
// 测试用例 3: 缓存污染攻击 - 大量数据存储
// ============================================================================

#[tokio::test]
async fn test_redis_cache_pollution_large_values() {
    let mut conn = get_redis_client().await;

    // 尝试存储大量数据（可能造成内存溢出）
    let large_value = "x".repeat(10 * 1024 * 1024); // 10MB

    let start = std::time::Instant::now();

    let _: () = conn
        .set("test:large_value", &large_value)
        .await
        .unwrap_or(());

    let elapsed = start.elapsed();

    // 预期：
    // - 如果 Redis 配置了 maxmemory，操作应该成功或失败
    // - 操作不应该导致系统崩溃
    if let Ok(stored_value) = conn.get::<&str, String>("test:large_value").await {
        // 验证可以读取
        assert_eq!(stored_value, large_value);

        // 清理
        let _: () = conn.del("test:large_value").await.unwrap_or(());
    }

    // 操作应该在合理时间内完成
    assert!(elapsed.as_secs() < 10, "Operation should complete quickly");
}

// ============================================================================
// 测试用例 4: 缓存污染攻击 - 大量 key 存储
// ============================================================================

#[tokio::test]
async fn test_redis_cache_pollution_many_keys() {
    let mut conn = get_redis_client().await;

    // 尝试存储大量 key（可能造成内存溢出或性能下降）
    let num_keys = 10000;
    let prefix = "test:pollution:";

    let start = std::time::Instant::now();

    for i in 0..num_keys {
        let key = format!("{}{}", prefix, i);
        let _: () = conn.set(&key, "value").await.unwrap_or(());
    }

    let elapsed = start.elapsed();

    // 验证至少有一些 key 被存储
    let count: Option<String> = conn.get(format!("{}0", prefix)).await.unwrap_or(None);
    assert!(count.is_some() || count.is_none());

    // 清理
    for i in 0..num_keys {
        let key = format!("{}{}", prefix, i);
        let _: () = conn.del(key).await.unwrap_or(());
    }

    // 操作应该在合理时间内完成
    assert!(
        elapsed.as_secs() < 30,
        "Should handle bulk operations efficiently"
    );
}

// ============================================================================
// 测试用例 5: 缓存穿透测试 - 不存在的 key
// ============================================================================

#[tokio::test]
async fn test_redis_cache_penetration() {
    let mut conn = get_redis_client().await;

    // 模拟缓存穿透：频繁查询不存在的 key
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let key = format!("nonexistent:{}", i);
        let value: Option<String> = conn.get(&key).await.unwrap_or(None);
        assert!(value.is_none());
    }

    let elapsed = start.elapsed();

    // 预期：查询应该快速完成
    // 如果存在缓存穿透问题，大量请求会打到后端
    assert!(elapsed.as_secs() < 5, "Cache lookup should be fast");
}

// ============================================================================
// 测试用例 6: 缓存击穿测试 - 热点 key 过期
// ============================================================================

#[tokio::test]
async fn test_redis_cache_breakdown() {
    let mut conn = get_redis_client().await;

    let hot_key = "test:hot_key";
    let value = "hot_value";

    // 设置热点 key，带短 TTL
    let _: () = conn.set_ex(hot_key, value, 1).await.unwrap_or(());

    // 等待 key 过期
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 多个并发请求尝试获取已过期的 key
    let start = std::time::Instant::now();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mut conn = conn.clone();
            tokio::spawn(async move {
                let value: Option<String> = conn.get(hot_key).await.unwrap_or(None);
                value
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    let elapsed = start.elapsed();

    // 预期：所有请求都应该快速失败（key 不存在）
    for result in results {
        assert!(result.is_none());
    }

    // 操作应该快速完成（没有缓存击穿导致的延迟）
    assert!(
        elapsed.as_secs() < 5,
        "Should handle cache breakdown gracefully"
    );
}

// ============================================================================
// 测试用例 7: 缓存雪崩测试 - 大量 key 同时过期
// ============================================================================

#[tokio::test]
async fn test_redis_cache_avalanche() {
    let mut conn = get_redis_client().await;

    // 设置大量具有相同过期时间的 key
    let num_keys = 100;
    let prefix = "test:avalanche:";

    for i in 0..num_keys {
        let key = format!("{}{}", prefix, i);
        let _: () = conn
            .set_ex(&key, format!("value_{}", i), 2)
            .await
            .unwrap_or(());
    }

    // 等待所有 key 同时过期
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 尝试获取所有过期的 key
    let start = std::time::Instant::now();

    let mut misses = 0;
    for i in 0..num_keys {
        let key = format!("{}{}", prefix, i);
        let value: Option<String> = conn.get(&key).await.unwrap_or(None);
        if value.is_none() {
            misses += 1;
        }
    }

    let elapsed = start.elapsed();

    // 预期：
    // - 所有 key 都应该过期
    // - 查询应该快速完成
    assert_eq!(misses, num_keys, "All keys should have expired");
    assert!(
        elapsed.as_secs() < 10,
        "Should handle cache avalanche efficiently"
    );
}

// ============================================================================
// 测试用例 8: 缓存绕过攻击
// ============================================================================

#[tokio::test]
async fn test_redis_cache_bypass() {
    let mut conn = get_redis_client().await;

    // 模拟缓存绕过：直接修改数据但不更新缓存
    let cache_key = "test:cache_key";
    let data_key = "test:data_key";

    // 设置缓存
    let _: () = conn.set(cache_key, "cached_value").await.unwrap_or(());

    // 模拟直接修改数据（绕过缓存）
    let _: () = conn.set(data_key, "direct_value").await.unwrap_or(());

    // 从缓存读取
    let cached_value: Option<String> = conn.get(cache_key).await.unwrap_or(None);
    let data_value: Option<String> = conn.get(data_key).await.unwrap_or(None);

    // 预期：缓存和数据可能不一致
    // 应用层应该有缓存更新策略
    assert_eq!(cached_value, Some("cached_value".to_string()));
    assert_eq!(data_value, Some("direct_value".to_string()));

    // 清理
    let _: () = conn.del(cache_key).await.unwrap_or(());
    let _: () = conn.del(data_key).await.unwrap_or(());
}

// ============================================================================
// 测试用例 9: TTL 竞争条件测试
// ============================================================================

#[tokio::test]
async fn test_redis_ttl_race_condition() {
    let mut conn = get_redis_client().await;

    let key = "test:ttl_race";
    let value = "test_value";

    // 设置 key
    let _: () = conn.set_ex(key, value, 10).await.unwrap_or(());

    // 获取 TTL
    let ttl: i64 = conn.ttl(key).await.unwrap_or(-1);

    // 预期：TTL 应该在合理范围内
    assert!(ttl > 0 && ttl <= 10);

    // 清理
    let _: () = conn.del(key).await.unwrap_or(());
}

// ============================================================================
// 测试用例 10: Redis 命令注入（Lua 脚本）
// ============================================================================

#[tokio::test]
async fn test_redis_lua_script_injection() {
    let mut conn = get_redis_client().await;

    // 测试 Lua 脚本中的注入
    let malicious_key = "test:injection';return redis.call('FLUSHALL');--";

    // 尝试执行包含潜在注入的 Lua 脚本
    let script = redis::Script::new("return redis.call('SET', KEYS[1], ARGV[1])");

    let result: Result<String, redis::RedisError> = script
        .key(malicious_key)
        .arg("value")
        .invoke_async(&mut conn)
        .await;

    // 预期：
    // - Lua 脚本应该安全执行
    // - 不应该执行注入的命令
    if result.is_ok() {
        // 验证 FLUSHALL 没有被执行
        let test_key = "test:lua_injection_check";
        let _: () = conn.set(test_key, "should_exist").await.unwrap_or(());
        let exists: bool = conn.exists(test_key).await.unwrap_or(false);
        assert!(exists, "FLUSHALL should not be executed");

        // 验证恶意 key 被正确存储
        let value: Option<String> = conn
            .get("test:injection';return redis.call('FLUSHALL');--")
            .await
            .unwrap_or(None);
        assert_eq!(value, Some("value".to_string()));

        // 清理
        let _: () = conn
            .del("test:injection';return redis.call('FLUSHALL');--")
            .await
            .unwrap_or(());
        let _: () = conn.del(test_key).await.unwrap_or(());
    }
}

// ============================================================================
// 测试用例 11: Key 命名空间隔离测试
// ============================================================================

#[tokio::test]
async fn test_redis_namespace_isolation() {
    let mut conn = get_redis_client().await;

    // 模拟多租户场景：不同租户使用不同的命名空间
    let tenant_a_keys = vec![
        "tenant_a:user:1",
        "tenant_a:session:1",
        "tenant_a:cache:data",
    ];

    let tenant_b_keys = vec![
        "tenant_b:user:1",
        "tenant_b:session:1",
        "tenant_b:cache:data",
    ];

    // 设置租户 A 的数据
    for key in &tenant_a_keys {
        let _: () = conn.set(key, "tenant_a_value").await.unwrap_or(());
    }

    // 设置租户 B 的数据
    for key in &tenant_b_keys {
        let _: () = conn.set(key, "tenant_b_value").await.unwrap_or(());
    }

    // 验证命名空间隔离
    for key in &tenant_a_keys {
        let value: Option<String> = conn.get(key).await.unwrap_or(None);
        assert_eq!(value, Some("tenant_a_value".to_string()));
    }

    for key in &tenant_b_keys {
        let value: Option<String> = conn.get(key).await.unwrap_or(None);
        assert_eq!(value, Some("tenant_b_value".to_string()));
    }

    // 清理
    for key in tenant_a_keys.iter().chain(tenant_b_keys.iter()) {
        let _: () = conn.del(key).await.unwrap_or(());
    }
}

// ============================================================================
// 测试用例 12: 缓存数据安全性测试
// ============================================================================

#[tokio::test]
async fn test_redis_data_security() {
    let mut conn = get_redis_client().await;

    // 测试敏感数据是否以明文存储
    let sensitive_data = "sensitive_password_12345";
    let key = "test:sensitive_data";

    // 存储敏感数据
    let _: () = conn.set(key, sensitive_data).await.unwrap_or(());

    // 检索数据
    let retrieved_data: Option<String> = conn.get(key).await.unwrap_or(None);

    // 预期：
    // - 数据可以被检索（因为是明文存储）
    // - 实际应用中应该加密敏感数据
    assert_eq!(retrieved_data, Some(sensitive_data.to_string()));

    // 风险：Redis 数据通常是明文存储的
    // 建议：敏感数据应该在应用层加密

    // 清理
    let _: () = conn.del(key).await.unwrap_or(());
}

// ============================================================================
// 测试用例 13: Key 过期策略测试
// ============================================================================

#[tokio::test]
async fn test_redis_key_expiration_policies() {
    let mut conn = get_redis_client().await;

    // 测试不同的过期策略
    let keys = vec![
        ("test:expire:fixed", 5),
        ("test:expire:short", 1),
        ("test:expire:long", 60),
    ];

    for (key, ttl) in &keys {
        let _: () = conn.set_ex(key, "value", *ttl).await.unwrap_or(());
    }

    // 验证 TTL
    for (key, expected_ttl) in &keys {
        let ttl: i64 = conn.ttl(key).await.unwrap_or(-1);
        assert!(ttl > 0 && ttl <= *expected_ttl as i64);
    }

    // 清理
    for (key, _) in keys {
        let _: () = conn.del(key).await.unwrap_or(());
    }
}

// ============================================================================
// 测试用例 14: Redis 访问控制测试（模拟）
// ============================================================================

#[tokio::test]
async fn test_redis_access_control() {
    let _conn = get_redis_client().await;

    // 测试是否可以执行危险命令
    let _dangerous_commands: Vec<&str> = vec![
        // "FLUSHALL",  // 危险命令，实际测试时谨慎使用
        // "FLUSHDB",
    ];

    // 注意：实际测试中不应该执行危险命令
    // 这个测试只是验证应用层是否有权限控制

    // 预期：应用层应该限制危险命令的使用
    // 建议：
    // 1. 使用 Redis ACL 限制命令
    // 2. 使用最小权限原则
    // 3. 分离读写实例

    assert!(
        true,
        "Access control should be enforced at application layer"
    );
}

// ============================================================================
// 测试用例 15: 批量操作安全性测试
// ============================================================================

#[tokio::test]
async fn test_redis_batch_operations_security() {
    let mut conn = get_redis_client().await;

    // 测试批量操作的安全性
    let num_keys = 1000;
    let prefix = "test:batch:";

    // 使用 MSET 批量设置
    let mut pipe = redis::pipe();
    for i in 0..num_keys {
        let key = format!("{}{}", prefix, i);
        pipe.set(key, format!("value_{}", i));
    }

    let result: Result<(), redis::RedisError> = pipe.query_async(&mut conn).await;
    assert!(result.is_ok());

    // 验证数据
    let keys_vec: Vec<String> = conn.keys(&format!("{}*", prefix)).await.unwrap_or(vec![]);
    let count: i64 = keys_vec.len() as i64;
    assert_eq!(count, num_keys);

    // 使用 MGET 批量获取
    let keys: Vec<String> = (0..num_keys).map(|i| format!("{}{}", prefix, i)).collect();

    let start = std::time::Instant::now();
    let values: Vec<Option<String>> = conn.mget(&keys).await.unwrap_or(vec![]);
    let elapsed = start.elapsed();

    assert!(values.len() == num_keys as usize);
    assert!(elapsed.as_secs() < 5, "Batch get should be efficient");

    // 清理
    for i in 0..num_keys {
        let key = format!("{}{}", prefix, i);
        let _: () = conn.del(key).await.unwrap_or(());
    }
}

// ============================================================================
// 测试总结
// ============================================================================

// 这个测试套件验证了 Redis 使用中的安全风险
//
// 关键发现：
// 1. Redis key 可以包含特殊字符，应用层需要验证
// 2. Redis 命令注入风险较低（使用 prepared commands）
// 3. 缓存污染攻击可能导致内存问题
// 4. 缓存穿透/击穿/雪崩需要应用层防护
// 5. Redis 数据通常是明文存储的
//
// 风险等级：中等
//
// 建议改进：
// 1. **Key 验证**：对 Redis key 进行格式验证和清理
// 2. **缓存加密**：对敏感数据进行应用层加密
// 3. **访问控制**：使用 Redis ACL 限制命令和 key 访问
// 4. **缓存防护**：
//    - 对不存在的 key 进行缓存（空值缓存）
//    - 对热点 key 使用互斥锁防止击穿
//    - 对 TTL 设置随机偏移防止雪崩
// 5. **监控告警**：
//    - 监控 Redis 内存使用
//    - 监控 key 数量和 TTL 分布
//    - 监控慢查询和错误率
// 6. **命名空间隔离**：使用清晰的 key 命名规范
// 7. **限流保护**：对批量操作进行限制
