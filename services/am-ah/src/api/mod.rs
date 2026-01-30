//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::AssetHierarchyServiceImpl;

pub mod proto {
    pub mod ah {
        tonic::include_proto!("am.ah.v1");
    }
}
