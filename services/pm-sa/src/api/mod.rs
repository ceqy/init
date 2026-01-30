//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::SourcingAnalyticsServiceImpl;

pub mod proto {
    pub mod sa {
        tonic::include_proto!("pm.sa.v1");
    }
}
