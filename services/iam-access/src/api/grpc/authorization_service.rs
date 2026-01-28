//! Authorization gRPC 服务实现

use std::sync::Arc;

use tonic::{Request, Response, Status};
use cuba_common::TenantId;

use crate::api::proto::authorization::{
    authorization_service_server::AuthorizationService as GrpcAuthorizationService,
    CheckRequest, CheckResponse,
    BatchCheckRequest, BatchCheckResponse,
    GetUserGrantedPermissionsRequest, GetUserGrantedPermissionsResponse,
    CheckResult as ProtoCheckResult,
};
use crate::application::{
    AuthorizationService, AuthorizationCheckRequest,
};
use crate::domain::policy::PolicyRepository;
use crate::domain::role::UserRoleRepository;

/// Authorization gRPC 服务
pub struct AuthorizationServiceImpl<PR, UR>
where
    PR: PolicyRepository + Send + Sync + 'static,
    UR: UserRoleRepository + Send + Sync + 'static,
{
    auth_service: AuthorizationService<PR, UR>,
}

impl<PR, UR> AuthorizationServiceImpl<PR, UR>
where
    PR: PolicyRepository + Send + Sync + 'static,
    UR: UserRoleRepository + Send + Sync + 'static,
{
    pub fn new(policy_repo: Arc<PR>, user_role_repo: Arc<UR>) -> Self {
        Self {
            auth_service: AuthorizationService::new(policy_repo, user_role_repo),
        }
    }
}

/// 从 subject 字符串中提取 user_id (例如 "user:uuid" -> "uuid")
fn extract_user_id(subject: &str) -> String {
    if let Some(id) = subject.strip_prefix("user:") {
        id.to_string()
    } else {
        subject.to_string()
    }
}

#[tonic::async_trait]
impl<PR, UR> GrpcAuthorizationService for AuthorizationServiceImpl<PR, UR>
where
    PR: PolicyRepository + Send + Sync + 'static,
    UR: UserRoleRepository + Send + Sync + 'static,
{
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req.tenant_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let user_id = extract_user_id(&req.subject);

        // 将 protobuf Struct 转换为 JSON 字符串
        let context = req.context.and_then(|c| {
            // 简单处理: 将 struct fields 转换为 JSON
            if c.fields.is_empty() {
                None
            } else {
                // 暂时使用 debug 格式
                Some(format!("{:?}", c))
            }
        });

        let check_request = AuthorizationCheckRequest {
            user_id,
            tenant_id,
            resource: req.resource,
            action: req.action,
            context,
        };

        let result = self.auth_service.check(check_request).await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(CheckResponse {
            allowed: result.allowed,
            reason: result.denied_reason.unwrap_or_default(),
        }))
    }

    async fn batch_check(
        &self,
        request: Request<BatchCheckRequest>,
    ) -> Result<Response<BatchCheckResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req.tenant_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let user_id = extract_user_id(&req.subject);

        // 将 protobuf Struct 转换为 JSON 字符串
        let context = req.context.and_then(|c| {
            if c.fields.is_empty() {
                None
            } else {
                Some(format!("{:?}", c))
            }
        });

        // 保存 checks 用于后续构建响应
        let checks_copy: Vec<_> = req.checks.iter()
            .map(|c| (c.resource.clone(), c.action.clone()))
            .collect();

        let requests: Vec<AuthorizationCheckRequest> = req.checks.into_iter().map(|c| {
            AuthorizationCheckRequest {
                user_id: user_id.clone(),
                tenant_id: tenant_id.clone(),
                resource: c.resource,
                action: c.action,
                context: context.clone(),
            }
        }).collect();

        let results = self.auth_service.batch_check(requests).await
            .map_err(|e| Status::from(e))?;

        let check_results: Vec<ProtoCheckResult> = checks_copy.iter().zip(results.iter()).map(|((resource, action), r)| {
            ProtoCheckResult {
                resource: resource.clone(),
                action: action.clone(),
                allowed: r.allowed,
            }
        }).collect();

        Ok(Response::new(BatchCheckResponse {
            results: check_results,
        }))
    }

    async fn get_user_granted_permissions(
        &self,
        request: Request<GetUserGrantedPermissionsRequest>,
    ) -> Result<Response<GetUserGrantedPermissionsResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req.tenant_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let permissions = self.auth_service
            .get_user_granted_permissions(&req.user_id, &tenant_id)
            .await
            .map_err(|e| Status::from(e))?;

        let permission_codes: Vec<String> = permissions.iter().map(|p| p.code.clone()).collect();

        // TODO: 获取角色代码
        let role_codes: Vec<String> = Vec::new();

        Ok(Response::new(GetUserGrantedPermissionsResponse {
            permission_codes,
            role_codes,
        }))
    }
}
