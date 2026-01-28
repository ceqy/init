//! Policy gRPC 服务实现

use cuba_common::TenantId;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::policy::{
    CreatePolicyRequest, CreatePolicyResponse, DeletePolicyRequest, DeletePolicyResponse,
    Effect as ProtoEffect, GetPolicyRequest, GetPolicyResponse, ListPoliciesRequest,
    ListPoliciesResponse, Policy as ProtoPolicy, UpdatePolicyRequest, UpdatePolicyResponse,
    policy_service_server::PolicyService as GrpcPolicyService,
};
use crate::application::policy::{
    PolicyCommandHandler,
    commands::{CreatePolicyCommand, DeletePolicyCommand, UpdatePolicyCommand},
};
use crate::domain::policy::{Effect, Policy, PolicyId, PolicyRepository};

/// Policy gRPC 服务实现
pub struct PolicyServiceImpl<R>
where
    R: PolicyRepository + Send + Sync + 'static,
{
    policy_cmd_handler: PolicyCommandHandler,
    policy_repo: Arc<R>,
}

impl<R> PolicyServiceImpl<R>
where
    R: PolicyRepository + Send + Sync + 'static,
{
    pub fn new(policy_cmd_handler: PolicyCommandHandler, policy_repo: Arc<R>) -> Self {
        Self {
            policy_cmd_handler,
            policy_repo,
        }
    }
}

fn policy_to_proto(policy: Policy) -> ProtoPolicy {
    ProtoPolicy {
        id: policy.id.to_string(),
        tenant_id: policy.tenant_id.to_string(),
        name: policy.name,
        description: policy.description.unwrap_or_default(),
        effect: match policy.effect {
            Effect::Allow => ProtoEffect::Allow as i32,
            Effect::Deny => ProtoEffect::Deny as i32,
        },
        subjects: policy.subjects,
        resources: policy.resources,
        actions: policy.actions,
        conditions: policy.conditions.unwrap_or_default(),
        priority: policy.priority,
        is_active: policy.is_active,
        created_at: Some(prost_types::Timestamp {
            seconds: policy.audit_info.created_at.timestamp(),
            nanos: policy.audit_info.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: policy.audit_info.updated_at.timestamp(),
            nanos: policy.audit_info.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

#[tonic::async_trait]
impl<R> GrpcPolicyService for PolicyServiceImpl<R>
where
    R: PolicyRepository + Send + Sync + 'static,
{
    async fn create_policy(
        &self,
        request: Request<CreatePolicyRequest>,
    ) -> Result<Response<CreatePolicyResponse>, Status> {
        let user_id = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .map(|u| u.user_id.clone())
            .unwrap_or_else(|| "system".to_string());

        let req = request.into_inner();

        let tenant_id = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let cmd = CreatePolicyCommand {
            tenant_id,
            name: req.name,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            effect: if req.effect == ProtoEffect::Deny as i32 {
                "DENY".to_string()
            } else {
                "ALLOW".to_string()
            },
            subjects: req.subjects,
            resources: req.resources,
            actions: req.actions,
            conditions: if req.conditions.is_empty() {
                None
            } else {
                Some(req.conditions)
            },
            priority: req.priority,
            performed_by: user_id,
        };

        let policy = self
            .policy_cmd_handler
            .handle_create(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(CreatePolicyResponse {
            policy: Some(policy_to_proto(policy)),
        }))
    }

    async fn update_policy(
        &self,
        request: Request<UpdatePolicyRequest>,
    ) -> Result<Response<UpdatePolicyResponse>, Status> {
        let user_id = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .map(|u| u.user_id.clone())
            .unwrap_or_else(|| "system".to_string());

        let req = request.into_inner();

        let cmd = UpdatePolicyCommand {
            policy_id: req.id,
            name: req.name,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            effect: if req.effect == ProtoEffect::Deny as i32 {
                "DENY".to_string()
            } else {
                "ALLOW".to_string()
            },
            subjects: req.subjects,
            resources: req.resources,
            actions: req.actions,
            conditions: if req.conditions.is_empty() {
                None
            } else {
                Some(req.conditions)
            },
            priority: req.priority,
            performed_by: user_id,
        };

        let policy = self
            .policy_cmd_handler
            .handle_update(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(UpdatePolicyResponse {
            policy: Some(policy_to_proto(policy)),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let user_id = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .map(|u| u.user_id.clone())
            .unwrap_or_else(|| "system".to_string());

        let req = request.into_inner();

        let cmd = DeletePolicyCommand {
            policy_id: req.id,
            performed_by: user_id,
        };

        self.policy_cmd_handler
            .handle_delete(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(DeletePolicyResponse { success: true }))
    }

    async fn get_policy(
        &self,
        request: Request<GetPolicyRequest>,
    ) -> Result<Response<GetPolicyResponse>, Status> {
        let req = request.into_inner();
        let id = req
            .id
            .parse::<PolicyId>()
            .map_err(|_| Status::invalid_argument("Invalid policy id"))?;

        let policy = self
            .policy_repo
            .find_by_id(&id)
            .await
            .map_err(|e| Status::from(e))?
            .ok_or_else(|| Status::not_found("Policy not found"))?;

        Ok(Response::new(GetPolicyResponse {
            policy: Some(policy_to_proto(policy)),
        }))
    }

    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = req
            .tenant_id
            .parse::<TenantId>()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let page = if req.page > 0 { req.page as u32 } else { 1 };
        let page_size = if req.page_size > 0 {
            req.page_size as u32
        } else {
            20
        };

        let (policies, total) = self
            .policy_repo
            .list_by_tenant(&tenant_id, page, page_size)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(ListPoliciesResponse {
            policies: policies.into_iter().map(policy_to_proto).collect(),
            total: total as i32,
            page: page as i32,
            page_size: page_size as i32,
        }))
    }

    async fn evaluate(
        &self,
        _request: Request<crate::api::proto::policy::EvaluateRequest>,
    ) -> Result<Response<crate::api::proto::policy::EvaluateResponse>, Status> {
        Err(Status::unimplemented(
            "Use AuthorizationService for evaluation",
        ))
    }

    async fn batch_evaluate(
        &self,
        _request: Request<crate::api::proto::policy::BatchEvaluateRequest>,
    ) -> Result<Response<crate::api::proto::policy::BatchEvaluateResponse>, Status> {
        Err(Status::unimplemented(
            "Use AuthorizationService for evaluation",
        ))
    }
}
