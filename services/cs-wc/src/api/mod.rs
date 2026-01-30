//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::WorkOrderServiceImpl;

pub mod proto {
    pub mod wc {
        tonic::include_proto!("cs.wc.v1");
    }
}
