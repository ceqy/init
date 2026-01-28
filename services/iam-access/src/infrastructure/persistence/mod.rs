//! 持久化层模块

pub mod db_metrics;
pub mod error_mapper;
pub mod outbox_repository;
pub mod outbox_publisher;
pub mod permission_repository;
pub mod policy_repository;
pub mod role_repository;
pub mod role_permission_repository;
pub mod user_role_repository;

pub use permission_repository::PostgresPermissionRepository;
pub use policy_repository::PostgresPolicyRepository;
pub use role_repository::PostgresRoleRepository;
pub use role_permission_repository::PostgresRolePermissionRepository;
pub use user_role_repository::PostgresUserRoleRepository;
pub use outbox_repository::{OutboxRepository, PostgresOutboxRepository, OutboxEvent};
pub use outbox_publisher::OutboxPublisher;
