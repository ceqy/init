//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::CostControlServiceImpl;

pub mod proto {
    pub mod co {
        tonic::include_proto!("fi.co.v1");
    }
}
