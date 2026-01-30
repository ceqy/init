//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::SourcingStrategyServiceImpl;

pub mod proto {
    pub mod se {
        tonic::include_proto!("pm.se.v1");
    }
}
