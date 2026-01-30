//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::OrderManagementServiceImpl;

pub mod proto {
    pub mod om {
        tonic::include_proto!("mf.om.v1");
    }
}
