//! AuditService gRPC 实现

use std::sync::Arc;

use prost_types::Timestamp;
use tonic::{Request, Response, Status};

use super::audit_proto::{self, audit_service_server::AuditService};
use crate::infrastructure::events::{EventQuery, EventStoreRepository, StoredEvent};

/// AuditService 实现
pub struct AuditServiceImpl {
    event_store_repo: Arc<dyn EventStoreRepository>,
}

impl AuditServiceImpl {
    pub fn new(event_store_repo: Arc<dyn EventStoreRepository>) -> Self {
        Self { event_store_repo }
    }

    fn stored_event_to_proto(event: StoredEvent) -> audit_proto::AuditEvent {
        audit_proto::AuditEvent {
            id: event.id.to_string(),
            event_type: event.event_type,
            aggregate_type: event.aggregate_type,
            aggregate_id: event.aggregate_id,
            tenant_id: event.tenant_id.to_string(),
            payload: event.payload.to_string(),
            created_at: Some(Timestamp {
                seconds: event.created_at.timestamp(),
                nanos: event.created_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

#[tonic::async_trait]
impl AuditService for AuditServiceImpl {
    async fn list_events(
        &self,
        request: Request<audit_proto::ListEventsRequest>,
    ) -> Result<Response<audit_proto::ListEventsResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(tenant_id = %req.tenant_id, "List audit events");

        let query = EventQuery {
            tenant_id: None, // 简化：暂不过滤
            event_type: if req.event_type.is_empty() {
                None
            } else {
                Some(req.event_type)
            },
            aggregate_type: if req.aggregate_type.is_empty() {
                None
            } else {
                Some(req.aggregate_type)
            },
            aggregate_id: if req.aggregate_id.is_empty() {
                None
            } else {
                Some(req.aggregate_id)
            },
            start_time: None,
            end_time: None,
            limit: if req.page_size > 0 {
                req.page_size as i64
            } else {
                50
            },
            offset: 0,
        };

        let events = self
            .event_store_repo
            .find_events(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total_count = self
            .event_store_repo
            .count_events(&EventQuery::default())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_events: Vec<audit_proto::AuditEvent> = events
            .into_iter()
            .map(Self::stored_event_to_proto)
            .collect();

        Ok(Response::new(audit_proto::ListEventsResponse {
            events: proto_events,
            next_page_token: String::new(),
            total_count: total_count as i32,
        }))
    }

    async fn get_user_audit_history(
        &self,
        request: Request<audit_proto::GetUserAuditHistoryRequest>,
    ) -> Result<Response<audit_proto::GetUserAuditHistoryResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(user_id = %req.user_id, "Get user audit history");

        let tenant_id = uuid::Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("Invalid tenant ID"))?;

        let events = self
            .event_store_repo
            .find_by_user_id(
                &req.user_id,
                tenant_id,
                if req.page_size > 0 {
                    req.page_size as i64
                } else {
                    50
                },
                0,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_events: Vec<audit_proto::AuditEvent> = events
            .into_iter()
            .map(Self::stored_event_to_proto)
            .collect();

        Ok(Response::new(audit_proto::GetUserAuditHistoryResponse {
            events: proto_events,
            next_page_token: String::new(),
        }))
    }
}
