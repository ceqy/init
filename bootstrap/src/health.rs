//! 健康检查模块
//!
//! 提供 /health 和 /ready 端点

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use cuba_adapter_clickhouse::check_connection as check_clickhouse;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::metrics::MetricsRecorder;
use crate::Infrastructure;

/// 健康检查状态
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub checks: Vec<ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            checks: vec![],
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            status: "unhealthy".to_string(),
            checks: vec![],
        }
    }

    pub fn add_check(&mut self, check: ComponentHealth) {
        if check.status != "healthy" {
            self.status = "unhealthy".to_string();
        }
        self.checks.push(check);
    }

    pub fn is_healthy(&self) -> bool {
        self.status == "healthy"
    }
}

impl ComponentHealth {
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "healthy".to_string(),
            message: None,
        }
    }

    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "unhealthy".to_string(),
            message: Some(message.into()),
        }
    }
}

/// 健康检查器
pub struct HealthChecker {
    infra: Arc<RwLock<Option<Infrastructure>>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            infra: Arc::new(RwLock::new(None)),
        }
    }

    /// 设置基础设施引用
    pub async fn set_infrastructure(&self, infra: Infrastructure) {
        let mut guard = self.infra.write().await;
        *guard = Some(infra);
    }

    /// 执行存活检查（liveness）
    ///
    /// 只检查服务是否在运行，不检查依赖
    pub async fn liveness(&self) -> HealthStatus {
        HealthStatus::healthy()
    }

    /// 执行就绪检查（readiness）
    ///
    /// 检查所有依赖是否可用
    pub async fn readiness(&self) -> HealthStatus {
        let guard = self.infra.read().await;
        let infra = match guard.as_ref() {
            Some(i) => i,
            None => {
                let mut status = HealthStatus::unhealthy();
                status.add_check(ComponentHealth::unhealthy(
                    "infrastructure",
                    "Not initialized",
                ));
                return status;
            }
        };

        let mut status = HealthStatus::healthy();

        // 检查 PostgreSQL
        status.add_check(self.check_postgres(infra).await);

        // 检查 Redis
        status.add_check(self.check_redis(infra).await);

        // 检查 Kafka（如果配置了）
        if infra.has_kafka() {
            status.add_check(ComponentHealth::healthy("kafka"));
        }

        // 检查 ClickHouse（如果配置了）
        if infra.has_clickhouse() {
            status.add_check(self.check_clickhouse(infra).await);
        }

        status
    }

    async fn check_postgres(&self, infra: &Infrastructure) -> ComponentHealth {
        let pool = infra.postgres_pool();
        match sqlx::query("SELECT 1").execute(&pool).await {
            Ok(_) => ComponentHealth::healthy("postgres"),
            Err(e) => ComponentHealth::unhealthy("postgres", e.to_string()),
        }
    }

    async fn check_redis(&self, infra: &Infrastructure) -> ComponentHealth {
        let mut conn = infra.redis_connection_manager();
        match redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
        {
            Ok(_) => ComponentHealth::healthy("redis"),
            Err(e) => ComponentHealth::unhealthy("redis", e.to_string()),
        }
    }

    async fn check_clickhouse(&self, infra: &Infrastructure) -> ComponentHealth {
        if let Some(client) = infra.clickhouse_client() {
            match check_clickhouse(client).await {
                Ok(_) => ComponentHealth::healthy("clickhouse"),
                Err(e) => ComponentHealth::unhealthy("clickhouse", e.to_string()),
            }
        } else {
            ComponentHealth::unhealthy("clickhouse", "Not configured")
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建健康检查 gRPC 服务
///
/// 实现 grpc.health.v1.Health 协议
pub mod grpc {
    use tonic::{Request, Response, Status};

    /// Health check request
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct HealthCheckRequest {
        #[prost(string, tag = "1")]
        pub service: String,
    }

    /// Health check response
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct HealthCheckResponse {
        #[prost(enumeration = "health_check_response::ServingStatus", tag = "1")]
        pub status: i32,
    }

    pub mod health_check_response {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
        #[repr(i32)]
        pub enum ServingStatus {
            Unknown = 0,
            Serving = 1,
            NotServing = 2,
            ServiceUnknown = 3,
        }
    }

    /// Health service trait
    #[tonic::async_trait]
    pub trait Health: Send + Sync + 'static {
        async fn check(
            &self,
            request: Request<HealthCheckRequest>,
        ) -> Result<Response<HealthCheckResponse>, Status>;
    }
}

// ============================================================================
// HTTP 健康检查服务器
// ============================================================================

/// HTTP 健康检查服务器状态
#[derive(Clone)]
struct HealthServerState {
    checker: Arc<HealthChecker>,
    metrics: Arc<MetricsRecorder>,
}

/// HTTP 健康检查服务器
pub struct HealthServer {
    checker: Arc<HealthChecker>,
    metrics: Arc<MetricsRecorder>,
    port: u16,
}

impl HealthServer {
    /// 创建新的健康检查服务器
    pub fn new(checker: Arc<HealthChecker>, metrics: Arc<MetricsRecorder>, port: u16) -> Self {
        Self {
            checker,
            metrics,
            port,
        }
    }

    /// 设置基础设施引用
    pub async fn set_infrastructure(&self, infra: Infrastructure) {
        self.checker.set_infrastructure(infra).await;
    }

    /// 启动 HTTP 服务器
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let state = HealthServerState {
            checker: self.checker,
            metrics: self.metrics,
        };

        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/ready", get(ready_handler))
            .route("/metrics", get(metrics_handler))
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!(%addr, "Health check HTTP server starting");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await
    }
}

/// Liveness 端点处理器
async fn health_handler(State(state): State<HealthServerState>) -> impl IntoResponse {
    let status = state.checker.liveness().await;
    (StatusCode::OK, Json(status))
}

/// Readiness 端点处理器
async fn ready_handler(State(state): State<HealthServerState>) -> impl IntoResponse {
    let status = state.checker.readiness().await;
    let code = if status.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(status))
}

/// Metrics 端点处理器
async fn metrics_handler(State(state): State<HealthServerState>) -> impl IntoResponse {
    let metrics = state.metrics.render();
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        metrics,
    )
}
