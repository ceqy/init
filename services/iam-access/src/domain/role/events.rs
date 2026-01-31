// use common::{TenantId, UserId}; // Removed unused
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RbacEvent {
    RoleCreated {
        id: Uuid,
        tenant_id: Uuid,
        code: String,
        name: String,
        by: Option<Uuid>,
    },
    RoleUpdated {
        id: Uuid,
        tenant_id: Uuid,
        by: Option<Uuid>,
    },
    RoleDeleted {
        id: Uuid,
        tenant_id: Uuid,
        by: Option<Uuid>,
    },
    RolePermissionsAssigned {
        role_id: Uuid,
        permission_ids: Vec<Uuid>,
        by: Option<Uuid>,
    },
    RolePermissionsRemoved {
        role_id: Uuid,
        permission_ids: Vec<Uuid>,
        by: Option<Uuid>,
    },
    UserRolesAssigned {
        user_id: Uuid,
        tenant_id: Uuid,
        role_ids: Vec<Uuid>,
        by: Option<Uuid>,
    },
    UserRolesRemoved {
        user_id: Uuid,
        tenant_id: Uuid,
        role_ids: Vec<Uuid>,
        by: Option<Uuid>,
    },
}

impl RbacEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            RbacEvent::RoleCreated { .. } => "RoleCreated",
            RbacEvent::RoleUpdated { .. } => "RoleUpdated",
            RbacEvent::RoleDeleted { .. } => "RoleDeleted",
            RbacEvent::RolePermissionsAssigned { .. } => "RolePermissionsAssigned",
            RbacEvent::RolePermissionsRemoved { .. } => "RolePermissionsRemoved",
            RbacEvent::UserRolesAssigned { .. } => "UserRolesAssigned",
            RbacEvent::UserRolesRemoved { .. } => "UserRolesRemoved",
        }
    }
}
