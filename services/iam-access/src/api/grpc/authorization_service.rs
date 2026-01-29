//! Authorization gRPC 服务实现

use std::sync::Arc;

use cuba_common::TenantId;
use tonic::{Request, Response, Status};

use crate::api::proto::authorization::{
    BatchCheckRequest, BatchCheckResponse, CheckRequest, CheckResponse,
    CheckResult as ProtoCheckResult, GetUserGrantedPermissionsRequest,
    GetUserGrantedPermissionsResponse,
    authorization_service_server::AuthorizationService as GrpcAuthorizationService,
};
use crate::application::{AuthorizationCheckRequest, AuthorizationService};
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
        // 创建追踪 Span
        let span = crate::api::grpc::create_request_span(&request, "CheckPermission");
        let _enter = span.enter();

        let req = request.into_inner();

        tracing::info!(
            tenant_id = %req.tenant_id,
            subject = %req.subject,
            resource = %req.resource,
            action = %req.action,
            "Authorization check request received"
        );

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let user_id = extract_user_id(&req.subject);

        // 将 protobuf Struct 转换为 JSON 字符串
        let context = req
            .context
            .map(|c| serde_json::to_string(&struct_to_json(c)).unwrap_or_default());

        let check_request = AuthorizationCheckRequest {
            user_id,
            tenant_id,
            resource: req.resource,
            action: req.action,
            context,
        };

        let result = self
            .auth_service
            .check(check_request)
            .await
            .map_err(Status::from)?;

        tracing::info!(
            allowed = result.allowed,
            decision_source = ?result.decision_source,
            "Authorization check finished"
        );

        Ok(Response::new(CheckResponse {
            allowed: result.allowed,
            reason: result.denied_reason.unwrap_or_default(),
        }))
    }

    async fn batch_check(
        &self,
        request: Request<BatchCheckRequest>,
    ) -> Result<Response<BatchCheckResponse>, Status> {
        let span = crate::api::grpc::create_request_span(&request, "BatchCheckPermissions");
        let _enter = span.enter();

        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let user_id = extract_user_id(&req.subject);

        // 将 protobuf Struct 转换为 JSON 字符串
        let context = req
            .context
            .map(|c| serde_json::to_string(&struct_to_json(c)).unwrap_or_default());

        // 保存 checks 用于后续构建响应
        let checks_copy: Vec<_> = req
            .checks
            .iter()
            .map(|c| (c.resource.clone(), c.action.clone()))
            .collect();

        let requests: Vec<AuthorizationCheckRequest> = req
            .checks
            .into_iter()
            .map(|c| AuthorizationCheckRequest {
                user_id: user_id.clone(),
                tenant_id: tenant_id.clone(),
                resource: c.resource,
                action: c.action,
                context: context.clone(),
            })
            .collect();

        let results = self
            .auth_service
            .batch_check(requests)
            .await
            .map_err(Status::from)?;

        let check_results: Vec<ProtoCheckResult> = checks_copy
            .iter()
            .zip(results.iter())
            .map(|((resource, action), r)| ProtoCheckResult {
                resource: resource.clone(),
                action: action.clone(),
                allowed: r.allowed,
            })
            .collect();

        Ok(Response::new(BatchCheckResponse {
            results: check_results,
        }))
    }

    async fn get_user_granted_permissions(
        &self,
        request: Request<GetUserGrantedPermissionsRequest>,
    ) -> Result<Response<GetUserGrantedPermissionsResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let permissions = self
            .auth_service
            .get_user_granted_permissions(&req.user_id, &tenant_id)
            .await
            .map_err(Status::from)?;

        let permission_codes: Vec<String> = permissions.iter().map(|p| p.code.clone()).collect();

        // 获取用户角色代码
        let roles = self
            .auth_service
            .get_user_roles(&req.user_id, &tenant_id)
            .await
            .map_err(Status::from)?;
        let role_codes: Vec<String> = roles.iter().map(|r| r.code.clone()).collect();

        Ok(Response::new(GetUserGrantedPermissionsResponse {
            permission_codes,
            role_codes,
        }))
    }
}

// ============ 辅助函数 ============

fn struct_to_json(s: prost_types::Struct) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in s.fields {
        map.insert(k, value_to_json(v));
    }
    serde_json::Value::Object(map)
}

fn value_to_json(v: prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;
    match v.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::NumberValue(n)) => serde_json::Value::Number(
            serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
        ),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(Kind::StructValue(s)) => struct_to_json(s),
        Some(Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.into_iter().map(value_to_json).collect())
        }
        None => serde_json::Value::Null,
    }
}
