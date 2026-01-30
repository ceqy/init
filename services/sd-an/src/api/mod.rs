//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::SalesAnalyticsServiceImpl;

pub mod proto {
    pub mod an {
        tonic::include_proto!("sd.an.v1");
    }
}
