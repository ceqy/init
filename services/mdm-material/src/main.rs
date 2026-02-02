#![allow(dead_code)]
#![allow(unused_imports)]

//! mdm-material Service - Material Master Data

mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

// 引入生成的 proto 代码
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod mdm_material {
    pub mod v1 {
        tonic::include_proto!("mdm.material.v1");
    }
}

use std::sync::Arc;

use bootstrap::{Infrastructure, run_server};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;

use api::MaterialServiceImpl;
use application::ServiceHandler;
use infrastructure::persistence::{
    PostgresMaterialGroupRepository, PostgresMaterialRepository, PostgresMaterialTypeRepository,
};
use mdm_material::v1::material_service_server::MaterialServiceServer;

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("mdm_material_descriptor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server("config", |infra: Infrastructure, mut server| async move {
        info!("Initializing mdm-material Service...");

        let pool = infra.postgres_pool();
        let material_repo = Arc::new(PostgresMaterialRepository::new(pool.clone()));
        let group_repo = Arc::new(PostgresMaterialGroupRepository::new(pool.clone()));
        let type_repo = Arc::new(PostgresMaterialTypeRepository::new(pool));
        info!("Repositories initialized");

        let handler = Arc::new(ServiceHandler::new(material_repo, group_repo, type_repo));
        let service = MaterialServiceImpl::new(handler);

        let reflection_service = ReflectionBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        Ok(server
            .add_service(MaterialServiceServer::new(service))
            .add_service(reflection_service))
    })
    .await
}
