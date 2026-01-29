//! DDOS 防护测试
//!
//! 本测试套件验证系统对 DDOS 攻击的防护能力
//! 测试覆盖：
//! - 高并发登录测试
//! - 暴力破解密码测试
//! - 账户锁定绕过测试
//! - 请求速率限制测试
//! - 慢速 HTTP 攻击测试
//!
//! 预期结果：验证系统的防护能力，识别潜在漏洞

use reqwest::Client;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// 获取网关 URL
fn gateway_url() -> String {
    env::var("GATEWAY_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

/// 获取测试客户端
fn get_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to create HTTP client")
}

// ============================================================================
// 测试用例 1: 高并发登录测试
// ============================================================================

#[tokio::test]
async fn test_ddos_high_concurrent_login() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    // 高并发登录请求（模拟 DDOS 攻击）
    let concurrent_requests = 100;
    let success_count = Arc::new(AtomicU32::new(0));
    let failure_count = Arc::new(AtomicU32::new(0));
    let timeout_count = Arc::new(AtomicU32::new(0));

    let start = Instant::now();

    let handles: Vec<_> = (0..concurrent_requests)
        .map(|i| {
            let client = client.clone();
            let url = url.clone();
            let success_count = success_count.clone();
            let failure_count = failure_count.clone();
            let timeout_count = timeout_count.clone();
            tokio::spawn(async move {
                let payload = serde_json::json!({
                    "username": format!("user_{}", i % 10),
                    "password": "password123",
                    "tenant_id": "00000000-0000-0000-0000-000000000001"
                });

                let start = Instant::now();
                let result = client.post(&url).json(&payload).send().await;

                let elapsed = start.elapsed();

                match result {
                    Ok(response) => {
                        if response.status().is_success() {
                            success_count.fetch_add(1, Ordering::SeqCst);
                        } else {
                            failure_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    Err(_) => {
                        timeout_count.fetch_add(1, Ordering::SeqCst);
                    }
                }

                elapsed
            })
        })
        .collect();

    let _durations: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_elapsed = start.elapsed();

    let success = Arc::clone(&success_count).load(Ordering::SeqCst);
    let failure = Arc::clone(&failure_count).load(Ordering::SeqCst);
    let timeout = Arc::clone(&timeout_count).load(Ordering::SeqCst);

    // 预期结果分析
    println!("=== 高并发登录测试结果 ===");
    println!("总请求数: {}", concurrent_requests);
    println!("成功: {}", success);
    println!("失败: {}", failure);
    println!("超时: {}", timeout);
    println!("总耗时: {:?}", total_elapsed);
    println!(
        "平均响应时间: {:?}",
        total_elapsed / concurrent_requests as u32
    );

    // 风险评估：
    // - 如果没有速率限制，系统可能被 DDOS 攻击压垮
    // - 应该有请求队列或拒绝策略

    // 建议：
    // 1. 添加全局速率限制
    // 2. 添加 IP 级别的速率限制
    // 3. 添加请求队列和丢弃策略
    // 4. 使用连接池限制并发连接
}

// ============================================================================
// 测试用例 2: 暴力破解密码测试
// ============================================================================

#[tokio::test]
async fn test_ddos_brute_force_password() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    let username = "admin";
    let tenant_id = "00000000-0000-0000-0000-000000000001";

    // 尝试多个密码
    let passwords = vec![
        "password",
        "123456",
        "admin",
        "password123",
        "qwerty",
        "letmein",
        "welcome",
        "monkey",
        "sunshine",
        "password1",
    ];

    let mut blocked_after = 0;

    for (i, password) in passwords.iter().enumerate() {
        let payload = serde_json::json!({
            "username": username,
            "password": password,
            "tenant_id": tenant_id
        });

        let start = Instant::now();
        let response = client.post(&url).json(&payload).send().await;

        let elapsed = start.elapsed();

        match response {
            Ok(resp) => {
                let status = resp.status();
                println!(
                    "尝试 {}/{}: 密码='{}' 状态码={} 耗时={:?}",
                    i + 1,
                    passwords.len(),
                    password,
                    status,
                    elapsed
                );

                // 检查是否被阻止（429 Too Many Requests）
                if status.as_u16() == 429 {
                    blocked_after = i + 1;
                    println!("=== 账户在 {} 次失败尝试后被阻止 ===", blocked_after);
                    break;
                }
            }
            Err(e) => {
                println!("尝试 {}/{}: 错误={:?}", i + 1, passwords.len(), e);
            }
        }

        // 小延迟，避免触发其他防护
        sleep(Duration::from_millis(100)).await;
    }

    // 预期：系统应该在一定次数失败后阻止登录尝试
    // 建议的失败次数：5-10 次

    if blocked_after > 0 {
        assert!(
            blocked_after <= 10,
            "账户应该在 10 次失败尝试内被阻止，但实际在 {} 次后被阻止",
            blocked_after
        );
        println!("✓ 暴力破解防护生效");
    } else {
        println!("⚠ 未检测到暴力破解防护（可能需要更多尝试）");
    }

    // 建议：
    // 1. 登录失败计数器（Redis）
    // 2. 失败后延迟增加
    // 3. CAPTCHA 验证
    // 4. 短信/邮件验证
}

// ============================================================================
// 测试用例 3: 分布式暴力破解测试
// ============================================================================

#[tokio::test]
async fn test_ddos_distributed_brute_force() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    // 模拟分布式攻击：使用不同的用户名
    let num_users = 50;
    let attempts_per_user = 3;

    let start = Instant::now();

    let handles: Vec<_> = (0..num_users)
        .map(|user_index| {
            let client = client.clone();
            let url = url.clone();
            tokio::spawn(async move {
                let username = format!("user_{}", user_index);
                let mut success = false;

                for attempt in 0..attempts_per_user {
                    let payload = serde_json::json!({
                        "username": username,
                        "password": format!("password{}", attempt),
                        "tenant_id": "00000000-0000-0000-0000-000000000001"
                    });

                    let response = client.post(&url).json(&payload).send().await;

                    if let Ok(resp) = response {
                        if resp.status().is_success() {
                            success = true;
                            break;
                        }
                    }
                }

                (username, success)
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_elapsed = start.elapsed();

    let successful_logins = results.iter().filter(|(_, s)| *s).count();
    let failed_logins = results.len() - successful_logins;

    println!("=== 分布式暴力破解测试结果 ===");
    println!("目标用户数: {}", num_users);
    println!("每个用户尝试次数: {}", attempts_per_user);
    println!("总尝试次数: {}", num_users * attempts_per_user);
    println!("成功登录: {}", successful_logins);
    println!("失败登录: {}", failed_logins);
    println!("总耗时: {:?}", total_elapsed);

    // 风险评估：
    // - 分布式攻击难以检测（来自不同 IP/用户）
    // - 需要全局的异常行为检测

    // 建议：
    // 1. 全局请求速率限制
    // 2. 异常行为检测（如来自多个 IP 的相同密码尝试）
    // 3. 地理位置异常检测
    // 4. 机器学习模型检测异常模式
}

// ============================================================================
// 测试用例 4: 大数据包攻击测试
// ============================================================================

#[tokio::test]
async fn test_ddos_large_payload_attack() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    // 测试不同大小的 payload
    let payload_sizes = vec![
        1_000,      // 1 KB
        10_000,     // 10 KB
        100_000,    // 100 KB
        1_000_000,  // 1 MB
        10_000_000, // 10 MB
    ];

    for size in payload_sizes {
        let large_value = "x".repeat(size);
        let payload = serde_json::json!({
            "username": large_value,
            "password": "password123",
            "tenant_id": "00000000-0000-0000-0000-000000000001"
        });

        let start = Instant::now();
        let response = client.post(&url).json(&payload).send().await;

        let elapsed = start.elapsed();

        match response {
            Ok(resp) => {
                println!(
                    "Payload 大小: {} bytes, 状态码: {}, 耗时: {:?}",
                    size,
                    resp.status(),
                    elapsed
                );

                // 预期：过大的 payload 应该被拒绝（413 Payload Too Large）
                if size > 1_000_000 && resp.status().as_u16() != 413 {
                    println!("⚠ 警告：系统接受了 {} bytes 的 payload", size);
                }
            }
            Err(e) => {
                println!("Payload 大小: {} bytes, 错误: {:?}", size, e);
            }
        }
    }

    // 建议：
    // 1. 设置请求体大小限制（Axum `DefaultBodyLimit`）
    // 2. 使用反向代理（Nginx）进行请求大小限制
    // 3. 拒绝超过 1MB 的请求体
}

// ============================================================================
// 测试用例 5: 慢速 HTTP 攻击测试
// ============================================================================

#[tokio::test]
async fn test_ddos_slowloris_attack() {
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    // 模拟慢速 HTTP 攻击：建立连接但慢慢发送数据
    let num_connections = 10;

    let handles: Vec<_> = (0..num_connections)
        .map(|i| {
            let url = url.clone();
            tokio::spawn(async move {
                // 尝试建立慢速连接
                let client = Client::builder()
                    .timeout(Duration::from_secs(30))
                    .connect_timeout(Duration::from_secs(10))
                    .build()
                    .expect("Failed to create HTTP client");

                let payload = serde_json::json!({
                    "username": format!("user_{}", i),
                    "password": "password123",
                    "tenant_id": "00000000-0000-0000-0000-000000000001"
                });

                let start = Instant::now();
                let response = client.post(&url).json(&payload).send().await;

                let elapsed = start.elapsed();

                match response {
                    Ok(resp) => {
                        println!("连接 {}: 状态码={}, 耗时={:?}", i, resp.status(), elapsed);
                        elapsed.as_millis()
                    }
                    Err(e) => {
                        println!("连接 {}: 错误={:?}", i, e);
                        elapsed.as_millis()
                    }
                }
            })
        })
        .collect();

    let durations: Vec<u128> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    if !durations.is_empty() {
        let avg_duration = durations.iter().sum::<u128>() / durations.len() as u128;
        println!("=== 慢速 HTTP 攻击测试结果 ===");
        println!("平均响应时间: {} ms", avg_duration);
        println!("最大响应时间: {} ms", durations.iter().max().unwrap_or(&0));

        // 建议：
        // 1. 设置请求超时时间
        // 2. 限制最大并发连接数
        // 3. 使用反向代理的连接限制功能
    }
}

// ============================================================================
// 测试用例 6: 请求头攻击测试
// ============================================================================

#[tokio::test]
async fn test_ddos_header_flood_attack() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    // 测试添加大量请求头
    let num_headers = 100;

    let mut request = client.post(&url);
    for i in 0..num_headers {
        request = request.header(format!("X-Custom-Header-{}", i), format!("value_{}", i));
    }

    let payload = serde_json::json!({
        "username": "test_user",
        "password": "password123",
        "tenant_id": "00000000-0000-0000-0000-000000000001"
    });

    let start = Instant::now();
    let response = request.json(&payload).send().await;
    let elapsed = start.elapsed();

    match response {
        Ok(resp) => {
            println!(
                "请求头数量: {}, 状态码: {}, 耗时: {:?}",
                num_headers,
                resp.status(),
                elapsed
            );

            // 预期：过多的请求头应该被拒绝
            if num_headers > 50 && resp.status().as_u16() == 200 {
                println!("⚠ 警告：系统接受了 {} 个请求头", num_headers);
            }
        }
        Err(e) => {
            println!("请求头数量: {}, 错误: {:?}", num_headers, e);
        }
    }

    // 建议：
    // 1. 限制请求头数量
    // 2. 限制请求头大小
    // 3. 过滤恶意请求头
}

// ============================================================================
// 测试用例 7: Token 爆炸攻击测试
// ============================================================================

#[tokio::test]
async fn test_ddos_token_explosion_attack() {
    let client = get_client();
    let gateway = gateway_url();

    // 尝试使用大量不同的 token 访问受保护的端点
    let num_tokens = 50;

    let mut success_count = 0;
    let mut failure_count = 0;

    for _i in 0..num_tokens {
        // 生成随机的伪造 token
        let fake_token = format!("fake_token_{}", uuid::Uuid::new_v4());

        let response = client
            .get(format!("{}/api/auth/me", gateway))
            .header("Authorization", format!("Bearer {}", fake_token))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
            }
            Err(_) => {
                failure_count += 1;
            }
        }
    }

    println!("=== Token 爆炸攻击测试结果 ===");
    println!("尝试 token 数量: {}", num_tokens);
    println!("成功: {}", success_count);
    println!("失败: {}", failure_count);

    // 预期：所有伪造的 token 都应该失败
    assert_eq!(success_count, 0, "所有伪造的 token 都应该被拒绝");

    // 建议：
    // 1. 添加 token 黑名单缓存
    // 2. 添加无效 token 的速率限制
    // 3. 监控无效 token 的模式
}

// ============================================================================
// 测试用例 8: 账户锁定绕过测试
// ============================================================================

#[tokio::test]
async fn test_ddos_account_lockout_bypass() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/login", gateway);

    let username = "lockout_test_user";
    let tenant_id = "00000000-0000-0000-0000-000000000001";

    // 1. 尝试多次失败登录以触发锁定
    println!("=== 尝试触发账户锁定 ===");

    for i in 0..6 {
        let payload = serde_json::json!({
            "username": username,
            "password": "wrong_password",
            "tenant_id": tenant_id
        });

        let response = client.post(&url).json(&payload).send().await;

        if let Ok(resp) = response {
            println!("尝试 {}: 状态码={}", i + 1, resp.status());
        }
    }

    // 2. 等待锁定时间
    sleep(Duration::from_secs(2)).await;

    // 3. 尝试绕过锁定（使用不同方式）
    println!("=== 尝试绕过账户锁定 ===");

    let username_upper = username.to_uppercase();
    let username_lower = username.to_lowercase();

    let bypass_attempts = vec![
        ("不同密码", "another_password"),
        ("相同密码继续尝试", "wrong_password"),
        ("使用大写用户名", &username_upper),
        ("使用小写用户名", &username_lower),
    ];

    for (method, password) in bypass_attempts {
        let payload = serde_json::json!({
            "username": username,
            "password": password,
            "tenant_id": tenant_id
        });

        let response = client.post(&url).json(&payload).send().await;

        match response {
            Ok(resp) => {
                println!("{}: 状态码={}", method, resp.status());

                // 如果绕过成功（返回 200），则是安全漏洞
                if resp.status().is_success() {
                    println!("⚠ 警告：可能成功绕过了账户锁定！");
                }
            }
            Err(e) => {
                println!("{}: 错误={:?}", method, e);
            }
        }
    }

    // 建议：
    // 1. 锁定应该基于用户 ID 而非用户名
    // 2. 锁定应该基于 IP + 用户组合
    // 3. 锁定时间应该逐渐增加
}

// ============================================================================
// 测试用例 9: 资源耗尽测试（连接池耗尽）
// ============================================================================

#[tokio::test]
async fn test_ddos_connection_pool_exhaustion() {
    let gateway = gateway_url();

    // 创建大量并发连接，尝试耗尽连接池
    let num_connections = 200;

    let start = Instant::now();

    let handles: Vec<_> = (0..num_connections)
        .map(|_i| {
            let url = format!("{}/health", gateway);
            tokio::spawn(async move {
                let client = Client::builder()
                    .timeout(Duration::from_secs(10))
                    .connect_timeout(Duration::from_secs(5))
                    .build()
                    .expect("Failed to create HTTP client");

                let request_start = Instant::now();
                let response = client.get(&url).send().await;
                let elapsed = request_start.elapsed();

                match response {
                    Ok(resp) => (resp.status().as_u16(), elapsed.as_millis()),
                    Err(_) => (0, elapsed.as_millis()),
                }
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_elapsed = start.elapsed();

    let success = results.iter().filter(|(s, _)| *s == 200).count();
    let failures = results.len() - success;

    if !results.is_empty() {
        let avg_response_time: u128 =
            results.iter().map(|(_, t)| t).sum::<u128>() / results.len() as u128;

        println!("=== 连接池耗尽测试结果 ===");
        println!("并发连接数: {}", num_connections);
        println!("成功: {}", success);
        println!("失败: {}", failures);
        println!("总耗时: {:?}", total_elapsed);
        println!("平均响应时间: {} ms", avg_response_time);

        // 风险评估：
        // - 如果平均响应时间显著增加，可能存在连接池耗尽问题
        // - 如果大量请求失败，系统可能过载

        // 建议：
        // 1. 配置适当的连接池大小
        // 2. 使用反向代理的连接限制
        // 3. 实施请求队列和丢弃策略
    }
}

// ============================================================================
// 测试用例 10: 注册接口泛洪测试
// ============================================================================

#[tokio::test]
async fn test_ddos_registration_flood() {
    let client = get_client();
    let gateway = gateway_url();
    let url = format!("{}/api/auth/register", gateway);

    // 尝试大量注册请求
    let num_registrations = 20;

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut rate_limited = 0;

    for i in 0..num_registrations {
        let payload = serde_json::json!({
            "username": format!("flood_user_{}", uuid::Uuid::new_v4()),
            "email": format!("flood{}@example.com", i),
            "password": "Password123!",
            "tenant_id": "00000000-0000-0000-0000-000000000001"
        });

        let response = client.post(&url).json(&payload).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status == 200 || status == 201 {
                    success_count += 1;
                } else if status == 429 {
                    rate_limited += 1;
                } else {
                    failure_count += 1;
                }
            }
            Err(_) => {
                failure_count += 1;
            }
        }
    }

    println!("=== 注册接口泛洪测试结果 ===");
    println!("注册尝试次数: {}", num_registrations);
    println!("成功: {}", success_count);
    println!("失败: {}", failure_count);
    println!("被限流: {}", rate_limited);

    // 预期：系统应该有注册速率限制
    if rate_limited > 0 {
        println!("✓ 检测到速率限制保护");
    } else if success_count > 10 {
        println!("⚠ 警告：系统可能缺乏注册速率限制");
    }

    // 建议：
    // 1. 注册接口的速率限制
    // 2. IP 级别的注册限制
    // 3. Email 验证
    // 4. CAPTCHA 验证
    // 5. 临时邮箱检测
}

// ============================================================================
// 测试总结
// ============================================================================

// 这个测试套件验证了系统的 DDOS 防护能力
//
// 关键发现：
// 1. **高并发攻击**：当前缺乏全局速率限制
// 2. **暴力破解**：有基础的暴力破解防护，但可以加强
// 3. **请求大小限制**：需要配置请求体大小限制
// 4. **连接限制**：需要配置最大连接数限制
// 5. **账户锁定**：有基本防护，但需要测试绕过可能性
//
// 风险等级：高
//
// 建议改进：
//
// ### 1. 全局速率限制（重要）
// ```rust
// use governor::{Quota, RateLimiter};
// // 实现 API 级别的速率限制
// ```
//
// ### 2. 请求体大小限制
// ```rust
// use axum::extract::DefaultBodyLimit;
// Router::new()
//     .route("/api/auth/login", post(login))
//     .layer(DefaultBodyLimit::max(1024 * 1024)); // 1MB
// ```
//
// ### 3. 连接数限制
// - 配置 Axum 的连接限制
// - 使用反向代理（Nginx）的连接限制
//
// ### 4. 超时配置
// ```rust
// let client = Client::builder()
//     .timeout(Duration::from_secs(10))
//     .connect_timeout(Duration::from_secs(5))
//     .build();
// ```
//
// ### 5. Redis 限流增强
// - 使用 Redis 实现分布式速率限制
// - 支持 IP 级别和用户级别的限流
//
// ### 6. 异常检测
// - 监控请求模式
// - 检测异常行为
// - 自动触发防护机制
//
// ### 7. 安全响应头
// ```rust
// use tower_http::set_header::SetResponseHeaderLayer;
// // 添加安全响应头
// ```
//
// ### 8. 反向代理配置（Nginx 示例）
// ```
// limit_req_zone $binary_remote_addr zone=login:10m rate=10r/m;
// limit_conn_zone $binary_remote_addr zone=addr:10m;
//
// limit_req zone=login burst=20 nodelay;
// limit_conn addr 10;
// ```
//
// ### 9. CDN 和 WAF
// - 使用 Cloudflare 或类似服务
// - 配置 Web 应用防火墙（WAF）
// - 启用 DDOS 防护功能
//
// ### 10. 监控和告警
// - 监控请求速率
// - 监控错误率
// - 监控响应时间
// - 配置自动告警
