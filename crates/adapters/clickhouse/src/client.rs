//! ClickHouse 客户端

use clickhouse::Client;
use cuba_errors::{AppError, AppResult};

/// ClickHouse 配置
#[derive(Debug, Clone)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl ClickHouseConfig {
    pub fn new(url: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            database: database.into(),
            user: None,
            password: None,
        }
    }

    pub fn with_auth(mut self, user: impl Into<String>, password: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self.password = Some(password.into());
        self
    }
}

/// 创建 ClickHouse 客户端
pub fn create_client(config: &ClickHouseConfig) -> AppResult<Client> {
    let mut client = Client::default()
        .with_url(&config.url)
        .with_database(&config.database);

    if let (Some(user), Some(password)) = (&config.user, &config.password) {
        client = client.with_user(user).with_password(password);
    }

    Ok(client)
}

/// 检查 ClickHouse 连接
pub async fn check_connection(client: &Client) -> AppResult<()> {
    client
        .query("SELECT 1")
        .fetch_one::<u8>()
        .await
        .map_err(|e| AppError::database(format!("ClickHouse health check failed: {}", e)))?;
    Ok(())
}
