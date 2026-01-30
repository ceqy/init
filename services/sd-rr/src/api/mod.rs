//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ReturnServiceImpl;

pub mod proto {
    pub mod rr {
        tonic::include_proto!("sd.rr.v1");
    }
}
