//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ProjectServiceImpl;

pub mod proto {
    pub mod ps {
        tonic::include_proto!("rd.ps.v1");
    }
}
