//! 认证路由

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};
use crate::middleware::AuthClaims;
use crate::grpc::{self, GrpcClients};
use serde::{Deserialize, Serialize};
use tonic::{metadata::MetadataValue, Request};
use tracing::{error, info, debug};

pub fn auth_routes() -> Router<GrpcClients> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/auth/register", post(register))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: String,
}

fn default_tenant_id() -> String {
    "00000000-0000-0000-0000-000000000001".to_string()
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: String,
    pub token_type: String,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub status: String,
}

async fn login(
    State(clients): State<GrpcClients>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    info!(username = %req.username, tenant_id = %req.tenant_id, "Login request");

    let mut grpc_req = Request::new(grpc::auth::LoginRequest {
        username: req.username.clone(),
        password: req.password,
        tenant_id: req.tenant_id.clone(),
        device_info: "gateway".to_string(),
        ip_address: "127.0.0.1".to_string(),
    });

    // 添加 tenant-id metadata
    grpc_req.metadata_mut().insert(
        "tenant-id",
        MetadataValue::try_from(&req.tenant_id).map_err(|e| {
            error!("Failed to create metadata: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
        })?,
    );

    let mut client = clients.auth.clone();
    let response = client.login(grpc_req).await.map_err(|e| {
        error!("gRPC login failed: {}", e);
        let status = match e.code() {
            tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, e.message().to_string())
    })?;

    let resp = response.into_inner();
    
    let user_info = resp.user.map(|u| UserInfo {
        id: u.id,
        username: u.username,
        email: u.email,
        display_name: u.display_name,
        status: u.status,
    });

    Ok(Json(LoginResponse {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_in: resp.expires_in.to_string(),
        token_type: resp.token_type,
        user: user_info,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub user: Option<UserInfo>,
}

async fn register(
    State(clients): State<GrpcClients>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, (StatusCode, String)> {
    info!(username = %req.username, email = %req.email, "Register request");

    let mut grpc_req = Request::new(grpc::user::RegisterRequest {
        username: req.username,
        email: req.email,
        password: req.password,
        display_name: req.display_name.unwrap_or_default(),
        phone: String::new(),
        tenant_id: req.tenant_id.clone(),
        role_ids: vec![],
    });

    grpc_req.metadata_mut().insert(
        "tenant-id",
        MetadataValue::try_from(&req.tenant_id).map_err(|e| {
            error!("Failed to create metadata: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
        })?,
    );

    let mut client = clients.user.clone();
    let response = client.register(grpc_req).await.map_err(|e| {
        error!("gRPC register failed: {}", e);
        let status = match e.code() {
            tonic::Code::AlreadyExists => StatusCode::CONFLICT,
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, e.message().to_string())
    })?;

    let resp = response.into_inner();
    
    let user_info = resp.user.map(|u| UserInfo {
        id: u.id,
        username: u.username,
        email: u.email,
        display_name: u.display_name,
        status: u.status,
    });

    Ok(Json(RegisterResponse {
        user_id: resp.user_id,
        user: user_info,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub access_token: String,
    #[serde(default)]
    pub logout_all_devices: bool,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

async fn logout(
    State(clients): State<GrpcClients>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    info!("Logout request");

    let grpc_req = Request::new(grpc::auth::LogoutRequest {
        access_token: req.access_token,
        logout_all_devices: req.logout_all_devices,
    });

    let mut client = clients.auth.clone();
    let response = client.logout(grpc_req).await.map_err(|e| {
        error!("gRPC logout failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
    })?;

    let resp = response.into_inner();

    Ok(Json(SuccessResponse {
        success: resp.success,
        message: None,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: String,
}

async fn refresh_token(
    State(clients): State<GrpcClients>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, String)> {
    info!("Refresh token request");

    let grpc_req = Request::new(grpc::auth::RefreshTokenRequest {
        refresh_token: req.refresh_token,
    });

    let mut client = clients.auth.clone();
    let response = client.refresh_token(grpc_req).await.map_err(|e| {
        error!("gRPC refresh token failed: {}", e);
        let status = match e.code() {
            tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, e.message().to_string())
    })?;

    let resp = response.into_inner();

    Ok(Json(RefreshTokenResponse {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_in: resp.expires_in.to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// 获取当前用户信息
///
/// 需要认证（使用 AuthClaims 提取器）
pub async fn get_current_user(
    State(clients): State<GrpcClients>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    debug!(
        user_id = %claims.sub,
        "Getting current user information"
    );

    let mut grpc_req = Request::new(grpc::user::GetUserRequest {
        user_id: claims.sub.clone(),
    });

    // 添加 tenant-id metadata
    grpc_req.metadata_mut().insert(
        "tenant-id",
        MetadataValue::try_from(&claims.tenant_id).map_err(|e| {
            error!("Failed to create metadata: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
        })?,
    );

    let mut client = clients.user.clone();
    let response = client.get_user(grpc_req).await.map_err(|e| {
        error!("gRPC get_user failed: {}", e);
        let status = match e.code() {
            tonic::Code::NotFound => StatusCode::NOT_FOUND,
            tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, e.message().to_string())
    })?;

    let resp = response.into_inner();

    let user = resp.user.ok_or_else(|| {
        error!("User not found in response");
        (StatusCode::NOT_FOUND, "User not found".to_string())
    })?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        display_name: if user.display_name.is_empty() {
            None
        } else {
            Some(user.display_name)
        },
    }))
}
