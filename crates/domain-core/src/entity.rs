//! 实体基础 trait

use cuba_common::AuditInfo;

/// 实体 trait
pub trait Entity {
    type Id;

    fn id(&self) -> &Self::Id;
}

/// 聚合根 trait
pub trait AggregateRoot: Entity {
    fn audit_info(&self) -> &AuditInfo;
    fn audit_info_mut(&mut self) -> &mut AuditInfo;
}
