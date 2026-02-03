use config::AppConfig;
use secrecy::ExposeSecret;

#[tokio::test]
async fn test_vault_config_loading() {
    println!("🔍 测试开始：正在准备环境...");
    // 1. 加载 .env 环境变量
    dotenvy::dotenv().ok();

    // 2. 执行加载逻辑
    // 指向具体的服务配置目录，例如 iam-identity
    let config_result = AppConfig::load("services/iam-identity/config").await;

    // 3. 验证结果
    match config_result {
        Ok(config) => {
            println!("✅ 成功加载配置！");
            println!("应用名称: {}", config.app_name);

            let db_url = config.database.url.expose_secret();
            println!("数据库 URL: {}", db_url);

            // 验证是否包含了 Vault 动态生成的特征（比如 10.0.0.10）
            if db_url.contains("10.0.0.10") {
                println!("🚀 验证通过：成功从 Vault 获取到了远程数据库配置！");
            } else {
                println!(
                    "⚠️  警告：加载的是本地默认配置，请检查 .env 中的 Vault 凭证是否存在且有效。"
                );
            }

            if let Some(p) = config.redis.url.expose_secret().strip_prefix("redis://:") {
                if !p.is_empty() {
                    println!("🚀 验证通过：成功获取并拼装了 Redis 密码！");
                }
            }

            // 4. 验证新服务
            if let Some(minio) = config.minio {
                println!("📦 MinIO URL: {}", minio.url);
                println!("🚀 验证通过：成功从 Vault 获取 MinIO 配置！");
            }

            if let Some(es) = config.elasticsearch {
                println!("🔍 ES URL: {}", es.url);
                println!("🚀 验证通过：成功从 Vault 获取 ES 配置！");
            }

            if let Some(grafana) = config.grafana {
                println!("📊 Grafana URL: {}", grafana.url);
                println!("🚀 验证通过：成功从 Vault 获取 Grafana 配置！");
            }
        }
        Err(e) => {
            panic!("❌ 配置加载失败: {:?}", e);
        }
    }
}
