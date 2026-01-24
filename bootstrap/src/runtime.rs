//! 服务运行时

use cuba_config::AppConfig;
use cuba_telemetry::{init_tracing, init_tracing_json};
use tracing::info;

/// 服务运行时配置
pub struct RuntimeConfig {
    pub config_dir: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            config_dir: "config".to_string(),
        }
    }
}

/// 初始化服务运行时
pub fn init_runtime(config: &AppConfig) {
    // 初始化 tracing
    if config.is_production() {
        init_tracing_json(&config.telemetry.log_level);
    } else {
        init_tracing(&config.telemetry.log_level);
    }

    info!(
        app_name = %config.app_name,
        app_env = %config.app_env,
        "Runtime initialized"
    );
}

/// 等待关闭信号
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
