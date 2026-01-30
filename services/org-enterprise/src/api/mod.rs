//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::EnterpriseServiceImpl;

pub mod proto {
    pub mod enterprise {
        tonic::include_proto!("org.enterprise.v1");
    }
}
