//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::CaseServiceImpl;

pub mod proto {
    pub mod cb {
        tonic::include_proto!("cs.cb.v1");
    }
}
