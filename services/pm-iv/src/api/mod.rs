//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::InvoiceVerificationServiceImpl;

pub mod proto {
    pub mod iv {
        tonic::include_proto!("pm.iv.v1");
    }
}
