//! 认证路由

use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/auth/me", get(get_current_user))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

async fn login(Json(req): Json<LoginRequest>) -> Json<LoginResponse> {
    // TODO: 调用 iam-auth gRPC 服务
    Json(LoginResponse {
        access_token: "todo".to_string(),
        refresh_token: "todo".to_string(),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
    })
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub logout_all_devices: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

async fn logout(Json(_req): Json<LogoutRequest>) -> Json<SuccessResponse> {
    // TODO: 调用 iam-auth gRPC 服务
    Json(SuccessResponse {
        success: true,
        message: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

async fn refresh_token(Json(_req): Json<RefreshTokenRequest>) -> Json<LoginResponse> {
    // TODO: 调用 iam-auth gRPC 服务
    Json(LoginResponse {
        access_token: "todo".to_string(),
        refresh_token: "todo".to_string(),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
}

async fn get_current_user() -> Json<UserResponse> {
    // TODO: 从 token 中获取用户信息
    Json(UserResponse {
        id: "todo".to_string(),
        username: "todo".to_string(),
        email: "todo@example.com".to_string(),
        display_name: None,
    })
}
