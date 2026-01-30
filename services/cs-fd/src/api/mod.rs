//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::FeedbackServiceImpl;

pub mod proto {
    pub mod fd {
        tonic::include_proto!("cs.fd.v1");
    }
}
