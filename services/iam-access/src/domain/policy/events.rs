//! Policy 领域事件

use super::policy::{Policy, PolicyId};
use cuba_common::TenantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyEvent {
    Created(PolicyCreated),
    Updated(PolicyUpdated),
    Deleted(PolicyDeleted),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCreated {
    pub id: PolicyId,
    pub tenant_id: TenantId,
    pub name: String,
    pub effect: String,
    pub performed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdated {
    pub id: PolicyId,
    pub tenant_id: TenantId,
    pub name: String,
    pub performed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDeleted {
    pub id: PolicyId,
    pub tenant_id: TenantId,
    pub performed_by: String,
}

impl PolicyEvent {
    pub fn created(policy: &Policy, performed_by: &str) -> Self {
        Self::Created(PolicyCreated {
            id: policy.id.clone(),
            tenant_id: policy.tenant_id.clone(),
            name: policy.name.clone(),
            effect: policy.effect.to_string(),
            performed_by: performed_by.to_string(),
        })
    }

    pub fn updated(policy: &Policy, performed_by: &str) -> Self {
        Self::Updated(PolicyUpdated {
            id: policy.id.clone(),
            tenant_id: policy.tenant_id.clone(),
            name: policy.name.clone(),
            performed_by: performed_by.to_string(),
        })
    }

    pub fn deleted(id: PolicyId, tenant_id: TenantId, performed_by: &str) -> Self {
        Self::Deleted(PolicyDeleted {
            id,
            tenant_id,
            performed_by: performed_by.to_string(),
        })
    }
}
