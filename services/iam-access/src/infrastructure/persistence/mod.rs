//! 持久化层模块

pub mod circuit_breaker;
pub mod db_metrics;
pub mod error_mapper;
pub mod iam_access_unit_of_work;
pub mod outbox_publisher;
pub mod outbox_repository;
pub mod permission_repository;
pub mod policy_repository;
pub mod role_permission_repository;
pub mod role_repository;
pub mod tx_repositories;
pub mod user_role_repository;

pub use iam_access_unit_of_work::{PostgresUnitOfWork, PostgresUnitOfWorkFactory};
pub use outbox_publisher::OutboxPublisher;
pub use outbox_repository::{OutboxEvent, OutboxRepository, PostgresOutboxRepository};
pub use permission_repository::PostgresPermissionRepository;
pub use policy_repository::PostgresPolicyRepository;
pub use role_permission_repository::PostgresRolePermissionRepository;
pub use role_repository::PostgresRoleRepository;
pub use user_role_repository::PostgresUserRoleRepository;
