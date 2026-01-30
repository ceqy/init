//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::BatchTrackingServiceImpl;

pub mod proto {
    pub mod bt {
        tonic::include_proto!("sc.bt.v1");
    }
}
