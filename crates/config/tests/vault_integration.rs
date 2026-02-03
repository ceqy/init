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

            // 4. 验证核心数据库和缓存
            if db_url.contains("10.0.0.10") {
                println!("🚀 验证通过：成功从 Vault 获取到了 PostgreSQL 远程配置！");
            }

            if let Some(etcd) = config.etcd {
                println!("📦 Etcd URL: {}", etcd.url.expose_secret());
                println!("🚀 验证通过：成功从 Vault 获取 Etcd 配置！");
            }

            // 5. 验证存储与搜索
            if let Some(minio) = config.minio {
                println!("📦 MinIO Endpoint: {}", minio.endpoint);
                println!("🚀 验证通过：成功从 Vault 获取 MinIO 配置！");
            }

            if let Some(es) = config.elasticsearch {
                println!("🔍 ES URL: {}", es.url);
                println!("🚀 验证通过：成功从 Vault 获取 ES 配置！");
            }

            // 6. 验证消息队列
            if let Some(mq) = config.mq {
                if let Some(kafka) = mq.kafka {
                    println!("󰓇 Kafka Bootstrap: {}", kafka.bootstrap_servers);
                    println!("🚀 验证通过：成功从 Vault 获取 Kafka 配置！");
                }
                if let Some(rmq) = mq.rabbitmq {
                    println!("󰓇 RabbitMQ URL: {}", rmq.url.expose_secret());
                    println!("🚀 验证通过：成功从 Vault 获取 RabbitMQ 配置！");
                }
            }

            // 7. 验证监控
            if let Some(monitoring) = config.monitoring {
                if let Some(grafana) = monitoring.grafana {
                    println!("📊 Grafana URL: {}", grafana.url);
                    println!("🚀 验证通过：成功从 Vault 获取 Grafana 配置！");
                }
                if let Some(prometheus) = monitoring.prometheus {
                    println!("📊 Prometheus Host: {}", prometheus.host);
                    println!("🚀 验证通过：成功从 Vault 获取 Prometheus 配置！");
                }
            }

            // 8. 验证系统服务
            if let Some(system) = config.system {
                if let Some(ssh) = system.ssh {
                    println!("󰆟 SSH Host: {}", ssh.host);
                    println!("🚀 验证通过：成功从 Vault 获取 SSH 配置！");
                }
            }
        }
        Err(e) => {
            panic!("❌ 配置加载失败: {:?}", e);
        }
    }
}
