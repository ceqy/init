#![allow(dead_code)]
#![allow(unused_imports)]

//! sys-notify Service - Message Notification and Job Scheduling

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

// common 模块（解决 proto 引用问题）
mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

use std::sync::Arc;

use cuba_bootstrap::{Infrastructure, run_server};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;

use application::ServiceHandler;
use infrastructure::persistence::PostgresRepository;

/// File descriptor set for gRPC reflection
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("sys_notify_descriptor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server("config", |infra: Infrastructure, mut server| async move {
        info!("Initializing sys-notify Service...");

        let pool = infra.postgres_pool();

        // Initialize repository
        let repo = Arc::new(PostgresRepository::new(pool));
        info!("Repository initialized");

        // Initialize handler
        let _handler = Arc::new(ServiceHandler::new(repo));

        // Build reflection service
        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        // TODO: Register gRPC service when implemented
        Ok(server.add_service(reflection_service))
    })
    .await
}
