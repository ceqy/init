#![allow(dead_code)]
#![allow(unused_imports)]

//! mdm-bp Service - Business Partner Master Data

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

use std::sync::Arc;

use bootstrap::{Infrastructure, run_server};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;

use application::ServiceHandler;
use infrastructure::persistence::PostgresRepository;

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("mdm_bp_descriptor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server("config", |infra: Infrastructure, mut server| async move {
        info!("Initializing mdm-bp Service...");

        let pool = infra.postgres_pool();
        let repo = Arc::new(PostgresRepository::new(pool));
        info!("Repository initialized");

        let _handler = Arc::new(ServiceHandler::new(repo));

        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        Ok(server.add_service(reflection_service))
    })
    .await
}
