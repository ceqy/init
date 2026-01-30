//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::MaterialServiceImpl;

pub mod proto {
    pub mod material {
        tonic::include_proto!("mdm.material.v1");
    }
}
