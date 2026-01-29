//! RBAC gRPC 服务实现

use std::sync::Arc;

use cuba_common::TenantId;
use tonic::{Request, Response, Status};

use crate::api::proto::rbac::{
    AssignPermissionsToRoleRequest, AssignPermissionsToRoleResponse, AssignRolesToUserRequest,
    AssignRolesToUserResponse, CheckPermissionRequest, CheckPermissionResponse,
    CheckPermissionsRequest, CheckPermissionsResponse, CreatePermissionRequest,
    CreatePermissionResponse, CreateRoleRequest, CreateRoleResponse, DeletePermissionRequest,
    DeletePermissionResponse, DeleteRoleRequest, DeleteRoleResponse, GetPermissionRequest,
    GetPermissionResponse, GetRolePermissionsRequest, GetRolePermissionsResponse, GetRoleRequest,
    GetRoleResponse, GetUserPermissionsRequest, GetUserPermissionsResponse, GetUserRolesRequest,
    GetUserRolesResponse, ListPermissionsRequest, ListPermissionsResponse, ListRolesRequest,
    ListRolesResponse, Permission as ProtoPermission, RemovePermissionsFromRoleRequest,
    RemovePermissionsFromRoleResponse, RemoveRolesFromUserRequest, RemoveRolesFromUserResponse,
    Role as ProtoRole, UpdatePermissionRequest, UpdatePermissionResponse, UpdateRoleRequest,
    UpdateRoleResponse, rbac_service_server::RbacService,
};
use crate::application::{
    AssignPermissionsToRoleCommand, AssignRolesToUserCommand, CheckUserPermissionQuery,
    CreateRoleCommand, DeleteRoleCommand, GetRoleQuery, GetUserPermissionsQuery, GetUserRolesQuery,
    ListRolesQuery, RemovePermissionsFromRoleCommand, RemoveRolesFromUserCommand,
    RoleCommandHandler, RoleQueryHandler, SearchRolesQuery, UpdateRoleCommand,
};
use crate::domain::role::{
    Permission, PermissionRepository, Role, RoleRepository, UserRoleRepository,
};

/// RBAC gRPC 服务
pub struct RbacServiceImpl<R, P, UR>
where
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    UR: UserRoleRepository + Send + Sync + 'static,
{
    role_cmd_handler: RoleCommandHandler,
    role_query_handler: RoleQueryHandler<R, UR>,
    permission_repo: Arc<P>,
}

use crate::domain::unit_of_work::UnitOfWorkFactory;

// Removed unused import: use cuba_ports::EventPublisher;

impl<R, P, UR> RbacServiceImpl<R, P, UR>
where
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    UR: UserRoleRepository + Send + Sync + 'static,
{
    pub fn new(
        role_repo: Arc<R>,
        permission_repo: Arc<P>,
        user_role_repo: Arc<UR>,
        uow_factory: Arc<dyn UnitOfWorkFactory>,
    ) -> Self {
        Self {
            role_cmd_handler: RoleCommandHandler::new(uow_factory),
            role_query_handler: RoleQueryHandler::new(role_repo, user_role_repo),
            permission_repo,
        }
    }
}

// ============ 转换函数 ============

fn role_to_proto(role: &Role) -> ProtoRole {
    ProtoRole {
        id: role.id.to_string(),
        code: role.code.clone(),
        name: role.name.clone(),
        description: role.description.clone().unwrap_or_default(),
        tenant_id: role.tenant_id.to_string(),
        is_system: role.is_system,
        is_active: role.is_active,
        permissions: role.permissions.iter().map(permission_to_proto).collect(),
        created_at: Some(prost_types::Timestamp {
            seconds: role.audit_info.created_at.timestamp(),
            nanos: role.audit_info.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: role.audit_info.updated_at.timestamp(),
            nanos: role.audit_info.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn permission_to_proto(perm: &Permission) -> ProtoPermission {
    ProtoPermission {
        id: perm.id.to_string(),
        code: perm.code.clone(),
        name: perm.name.clone(),
        description: perm.description.clone().unwrap_or_default(),
        resource: perm.resource.clone(),
        action: perm.action.clone(),
        module: perm.module.clone(),
        is_active: perm.is_active,
        created_at: Some(prost_types::Timestamp {
            seconds: perm.created_at.timestamp(),
            nanos: perm.created_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

#[tonic::async_trait]
impl<R, P, UR> RbacService for RbacServiceImpl<R, P, UR>
where
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    UR: UserRoleRepository + Send + Sync + 'static,
{
    // ===== 角色管理 =====

    async fn create_role(
        &self,
        request: Request<CreateRoleRequest>,
    ) -> Result<Response<CreateRoleResponse>, Status> {
        let start = std::time::Instant::now();

        let user_id = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .map(|u| u.user_id.clone());
        let performed_by = user_id
            .as_ref()
            .and_then(|id| id.parse::<uuid::Uuid>().ok());
        let trace_id = request
            .extensions()
            .get::<crate::api::grpc::TraceInfo>()
            .map(|t| t.trace_id.clone());

        let req = request.into_inner();

        // 创建更丰富的追踪 Span
        let span = tracing::info_span!(
            "grpc_request",
            span_name = "CreateRole",
            tenant_id = %req.tenant_id,
            role_code = %req.code,
            trace_id = tracing::field::Empty
        );

        // 注入 Trace ID
        if let Some(tid) = trace_id {
            span.record("trace_id", &tid);
        }

        let _enter = span.enter();

        tracing::info!("Processing CreateRole request");

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let cmd = CreateRoleCommand {
            tenant_id,
            code: req.code.clone(),
            name: req.name.clone(),
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            is_system: false,
            performed_by,
        };

        let result = self.role_cmd_handler.handle_create(cmd).await;

        let duration = start.elapsed();

        match result {
            Ok(role) => {
                metrics::counter!("rbac_roles_created_total", "tenant_id" => req.tenant_id)
                    .increment(1);
                metrics::histogram!("rbac_request_duration_ms", "method" => "CreateRole")
                    .record(duration.as_millis() as f64);

                tracing::info!(role_id = %role.id.0, "Role created successfully");
                Ok(Response::new(CreateRoleResponse {
                    role: Some(role_to_proto(&role)),
                }))
            }
            Err(e) => {
                metrics::counter!("rbac_request_errors_total", "method" => "CreateRole", "error" => "conflict").increment(1);
                Err(Status::from(e))
            }
        }
    }

    async fn update_role(
        &self,
        request: Request<UpdateRoleRequest>,
    ) -> Result<Response<UpdateRoleResponse>, Status> {
        let performed_by = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .and_then(|u| u.user_id.parse::<uuid::Uuid>().ok());

        let req = request.into_inner();

        let cmd = UpdateRoleCommand {
            role_id: req.id,
            name: req.name,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            performed_by,
        };

        let role = self
            .role_cmd_handler
            .handle_update(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(UpdateRoleResponse {
            role: Some(role_to_proto(&role)),
        }))
    }

    async fn delete_role(
        &self,
        request: Request<DeleteRoleRequest>,
    ) -> Result<Response<DeleteRoleResponse>, Status> {
        let performed_by = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .and_then(|u| u.user_id.parse::<uuid::Uuid>().ok());

        let req = request.into_inner();

        let cmd = DeleteRoleCommand {
            role_id: req.id,
            performed_by,
        };

        self.role_cmd_handler
            .handle_delete(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(DeleteRoleResponse { success: true }))
    }

    async fn get_role(
        &self,
        request: Request<GetRoleRequest>,
    ) -> Result<Response<GetRoleResponse>, Status> {
        let req = request.into_inner();

        let query = GetRoleQuery { role_id: req.id };

        let role = self
            .role_query_handler
            .handle_get(query)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(GetRoleResponse {
            role: Some(role_to_proto(&role)),
        }))
    }

    async fn list_roles(
        &self,
        request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let result = if req.search.is_empty() {
            let query = ListRolesQuery {
                tenant_id,
                page: req.page.max(1) as u32,
                page_size: req.page_size.clamp(1, 100) as u32,
            };
            self.role_query_handler.handle_list(query).await
        } else {
            let query = SearchRolesQuery {
                tenant_id,
                query: req.search,
                page: req.page.max(1) as u32,
                page_size: req.page_size.clamp(1, 100) as u32,
            };
            self.role_query_handler.handle_search(query).await
        }
        .map_err(|e| Status::from(e))?;

        Ok(Response::new(ListRolesResponse {
            roles: result.roles.iter().map(role_to_proto).collect(),
            total: result.total as i32,
            page: result.page as i32,
            page_size: result.page_size as i32,
            next_cursor: result
                .roles
                .last()
                .map(|r| r.id.to_string())
                .unwrap_or_default(),
        }))
    }

    // ===== 权限管理 =====

    async fn create_permission(
        &self,
        request: Request<CreatePermissionRequest>,
    ) -> Result<Response<CreatePermissionResponse>, Status> {
        let req = request.into_inner();

        let permission = Permission::new(
            req.code,
            req.name,
            if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            req.resource,
            req.action,
            req.module,
        );

        self.permission_repo
            .create(&permission)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(CreatePermissionResponse {
            permission: Some(permission_to_proto(&permission)),
        }))
    }

    async fn update_permission(
        &self,
        request: Request<UpdatePermissionRequest>,
    ) -> Result<Response<UpdatePermissionResponse>, Status> {
        let req = request.into_inner();

        let perm_id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid permission ID"))?;

        let mut permission = self
            .permission_repo
            .find_by_id(&perm_id)
            .await
            .map_err(|e| Status::from(e))?
            .ok_or_else(|| Status::not_found("Permission not found"))?;

        permission.name = req.name;
        permission.description = if req.description.is_empty() {
            None
        } else {
            Some(req.description)
        };
        if req.is_active {
            permission.activate();
        } else {
            permission.deactivate();
        }

        self.permission_repo
            .update(&permission)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(UpdatePermissionResponse {
            permission: Some(permission_to_proto(&permission)),
        }))
    }

    async fn delete_permission(
        &self,
        request: Request<DeletePermissionRequest>,
    ) -> Result<Response<DeletePermissionResponse>, Status> {
        let req = request.into_inner();

        let perm_id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid permission ID"))?;

        self.permission_repo
            .delete(&perm_id)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(DeletePermissionResponse { success: true }))
    }

    async fn get_permission(
        &self,
        request: Request<GetPermissionRequest>,
    ) -> Result<Response<GetPermissionResponse>, Status> {
        let req = request.into_inner();

        let perm_id = req
            .id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid permission ID"))?;

        let permission = self
            .permission_repo
            .find_by_id(&perm_id)
            .await
            .map_err(|e| Status::from(e))?
            .ok_or_else(|| Status::not_found("Permission not found"))?;

        Ok(Response::new(GetPermissionResponse {
            permission: Some(permission_to_proto(&permission)),
        }))
    }

    async fn list_permissions(
        &self,
        request: Request<ListPermissionsRequest>,
    ) -> Result<Response<ListPermissionsResponse>, Status> {
        let req = request.into_inner();

        let (permissions, total) = if !req.module.is_empty() {
            let perms = self
                .permission_repo
                .list_by_module(&req.module)
                .await
                .map_err(|e| Status::from(e))?;
            let len = perms.len() as i64;
            (perms, len)
        } else if !req.resource.is_empty() {
            let perms = self
                .permission_repo
                .list_by_resource(&req.resource)
                .await
                .map_err(|e| Status::from(e))?;
            let len = perms.len() as i64;
            (perms, len)
        } else {
            self.permission_repo
                .list_all(req.page.max(1) as u32, req.page_size.clamp(1, 100) as u32)
                .await
                .map_err(|e| Status::from(e))?
        };

        Ok(Response::new(ListPermissionsResponse {
            permissions: permissions.iter().map(permission_to_proto).collect(),
            total: total as i32,
        }))
    }

    // ===== 角色权限关联 =====

    async fn assign_permissions_to_role(
        &self,
        request: Request<AssignPermissionsToRoleRequest>,
    ) -> Result<Response<AssignPermissionsToRoleResponse>, Status> {
        let performed_by = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .and_then(|u| u.user_id.parse::<uuid::Uuid>().ok());

        let req = request.into_inner();

        let cmd = AssignPermissionsToRoleCommand {
            role_id: req.role_id,
            permission_ids: req.permission_ids,
            performed_by,
        };

        self.role_cmd_handler
            .handle_assign_permissions(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(AssignPermissionsToRoleResponse {
            success: true,
        }))
    }

    async fn remove_permissions_from_role(
        &self,
        request: Request<RemovePermissionsFromRoleRequest>,
    ) -> Result<Response<RemovePermissionsFromRoleResponse>, Status> {
        let performed_by = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .and_then(|u| u.user_id.parse::<uuid::Uuid>().ok());

        let req = request.into_inner();

        let cmd = RemovePermissionsFromRoleCommand {
            role_id: req.role_id,
            permission_ids: req.permission_ids,
            performed_by,
        };

        self.role_cmd_handler
            .handle_remove_permissions(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(RemovePermissionsFromRoleResponse {
            success: true,
        }))
    }

    async fn get_role_permissions(
        &self,
        request: Request<GetRolePermissionsRequest>,
    ) -> Result<Response<GetRolePermissionsResponse>, Status> {
        let req = request.into_inner();

        let query = GetRoleQuery {
            role_id: req.role_id,
        };

        let role = self
            .role_query_handler
            .handle_get(query)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(GetRolePermissionsResponse {
            permissions: role.permissions.iter().map(permission_to_proto).collect(),
        }))
    }

    // ===== 用户角色关联 =====

    async fn assign_roles_to_user(
        &self,
        request: Request<AssignRolesToUserRequest>,
    ) -> Result<Response<AssignRolesToUserResponse>, Status> {
        let performed_by = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .and_then(|u| u.user_id.parse::<uuid::Uuid>().ok());

        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let cmd = AssignRolesToUserCommand {
            user_id: req.user_id,
            tenant_id,
            role_ids: req.role_ids,
            performed_by,
        };

        self.role_cmd_handler
            .handle_assign_roles_to_user(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(AssignRolesToUserResponse { success: true }))
    }

    async fn remove_roles_from_user(
        &self,
        request: Request<RemoveRolesFromUserRequest>,
    ) -> Result<Response<RemoveRolesFromUserResponse>, Status> {
        let performed_by = request
            .extensions()
            .get::<crate::api::grpc::interceptor::UserInfo>()
            .and_then(|u| u.user_id.parse::<uuid::Uuid>().ok());

        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let cmd = RemoveRolesFromUserCommand {
            user_id: req.user_id,
            tenant_id,
            role_ids: req.role_ids,
            performed_by,
        };

        self.role_cmd_handler
            .handle_remove_roles_from_user(cmd)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(RemoveRolesFromUserResponse { success: true }))
    }

    async fn get_user_roles(
        &self,
        request: Request<GetUserRolesRequest>,
    ) -> Result<Response<GetUserRolesResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let query = GetUserRolesQuery {
            user_id: req.user_id,
            tenant_id,
        };

        let roles = self
            .role_query_handler
            .handle_get_user_roles(query)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(GetUserRolesResponse {
            roles: roles.iter().map(role_to_proto).collect(),
        }))
    }

    async fn get_user_permissions(
        &self,
        request: Request<GetUserPermissionsRequest>,
    ) -> Result<Response<GetUserPermissionsResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let query = GetUserPermissionsQuery {
            user_id: req.user_id,
            tenant_id,
        };

        let permissions = self
            .role_query_handler
            .handle_get_user_permissions(query)
            .await
            .map_err(|e| Status::from(e))?;

        let permission_codes: Vec<String> = permissions.iter().map(|p| p.code.clone()).collect();

        Ok(Response::new(GetUserPermissionsResponse {
            permissions: permissions.iter().map(permission_to_proto).collect(),
            permission_codes,
        }))
    }

    // ===== 权限检查 =====

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let query = CheckUserPermissionQuery {
            user_id: req.user_id,
            tenant_id,
            permission_code: req.permission_code,
        };

        let allowed = self
            .role_query_handler
            .handle_check_user_permission(query)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(CheckPermissionResponse { allowed }))
    }

    async fn check_permissions(
        &self,
        request: Request<CheckPermissionsRequest>,
    ) -> Result<Response<CheckPermissionsResponse>, Status> {
        let req = request.into_inner();

        let tenant_id: TenantId = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid tenant_id"))?;

        let mut results = std::collections::HashMap::new();

        for code in req.permission_codes {
            let query = CheckUserPermissionQuery {
                user_id: req.user_id.clone(),
                tenant_id: tenant_id.clone(),
                permission_code: code.clone(),
            };

            let allowed = self
                .role_query_handler
                .handle_check_user_permission(query)
                .await
                .map_err(|e| Status::from(e))?;

            results.insert(code, allowed);
        }

        Ok(Response::new(CheckPermissionsResponse { results }))
    }

    // ===== 导入导出 (待实现) =====

    type ExportRolesStream = std::pin::Pin<
        Box<dyn futures::Stream<Item = Result<crate::api::proto::rbac::Role, Status>> + Send>,
    >;

    async fn export_roles(
        &self,
        _request: Request<crate::api::proto::rbac::ExportRolesRequest>,
    ) -> Result<Response<Self::ExportRolesStream>, Status> {
        // TODO: Implement streaming export
        Err(Status::unimplemented("ExportRoles is not yet implemented"))
    }

    async fn import_roles(
        &self,
        _request: Request<tonic::Streaming<crate::api::proto::rbac::ImportRoleRequest>>,
    ) -> Result<Response<crate::api::proto::rbac::ImportRolesResponse>, Status> {
        // TODO: Implement streaming import
        Err(Status::unimplemented("ImportRoles is not yet implemented"))
    }
}
