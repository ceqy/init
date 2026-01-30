//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ContractServiceImpl;

pub mod proto {
    pub mod ct {
        tonic::include_proto!("pm.ct.v1");
    }
}
