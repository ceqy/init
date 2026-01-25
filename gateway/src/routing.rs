//! API 路由

use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn api_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub checks: Vec<ServiceCheck>,
}

#[derive(Debug, Serialize)]
pub struct ServiceCheck {
    pub name: String,
    pub healthy: bool,
}

async fn readiness_check() -> Json<ReadinessResponse> {
    // TODO: 检查各个服务的连接状态
    Json(ReadinessResponse {
        ready: true,
        checks: vec![
            ServiceCheck {
                name: "iam-identity".to_string(),
                healthy: true,
            },
        ],
    })
}
