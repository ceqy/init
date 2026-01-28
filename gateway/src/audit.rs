//! 审计路由

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use crate::grpc::{self, GrpcClients};
use crate::middleware::AuthToken;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// 审计路由
pub fn audit_routes() -> Router<GrpcClients> {
    Router::new()
        .route("/events", get(list_events))
        .route("/user-history", get(get_user_audit_history))
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub page_size: Option<i32>,
    pub event_type: Option<String>,
    pub aggregate_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditEventResponse {
    pub id: String,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub tenant_id: String,
    pub payload: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListEventsResponse {
    pub events: Vec<AuditEventResponse>,
    pub total_count: i32,
}

/// 列出审计事件
async fn list_events(
    State(clients): State<GrpcClients>,
    AuthToken(auth_context): AuthToken,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<ListEventsResponse>, (StatusCode, String)> {
    info!("List audit events request");

    let mut client = clients.audit.clone();
    let request = grpc::audit::ListEventsRequest {
        tenant_id: auth_context.claims.tenant_id.clone(),
        page_size: query.page_size.unwrap_or(50),
        event_type: query.event_type.unwrap_or_default(),
        aggregate_type: query.aggregate_type.unwrap_or_default(),
        aggregate_id: String::new(),
        start_time: None,
        end_time: None,
        page_token: String::new(),
    };

    let response = client
        .list_events(request)
        .await
        .map_err(|e| {
            error!("Failed to list events: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
        })?;

    let inner = response.into_inner();
    let events: Vec<AuditEventResponse> = inner
        .events
        .into_iter()
        .map(|e| AuditEventResponse {
            id: e.id,
            event_type: e.event_type,
            aggregate_type: e.aggregate_type,
            aggregate_id: e.aggregate_id,
            tenant_id: e.tenant_id,
            payload: e.payload,
            created_at: e.created_at.map(|t| {
                chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            }),
        })
        .collect();

    Ok(Json(ListEventsResponse {
        events,
        total_count: inner.total_count,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UserAuditHistoryQuery {
    pub user_id: String,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct UserAuditHistoryResponse {
    pub events: Vec<AuditEventResponse>,
}

/// 获取用户审计历史
async fn get_user_audit_history(
    State(clients): State<GrpcClients>,
    AuthToken(auth_context): AuthToken,
    Query(query): Query<UserAuditHistoryQuery>,
) -> Result<Json<UserAuditHistoryResponse>, (StatusCode, String)> {
    info!(user_id = %query.user_id, "Get user audit history request");

    let mut client = clients.audit.clone();
    let request = grpc::audit::GetUserAuditHistoryRequest {
        user_id: query.user_id,
        tenant_id: auth_context.claims.tenant_id.clone(),
        page_size: query.page_size.unwrap_or(50),
        page_token: String::new(),
    };

    let response = client
        .get_user_audit_history(request)
        .await
        .map_err(|e| {
            error!("Failed to get user audit history: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
        })?;

    let inner = response.into_inner();
    let events: Vec<AuditEventResponse> = inner
        .events
        .into_iter()
        .map(|e| AuditEventResponse {
            id: e.id,
            event_type: e.event_type,
            aggregate_type: e.aggregate_type,
            aggregate_id: e.aggregate_id,
            tenant_id: e.tenant_id,
            payload: e.payload,
            created_at: e.created_at.map(|t| {
                chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            }),
        })
        .collect();

    Ok(Json(UserAuditHistoryResponse { events }))
}
