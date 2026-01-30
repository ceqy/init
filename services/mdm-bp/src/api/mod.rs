//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::BusinessPartnerServiceImpl;

pub mod proto {
    pub mod bp {
        tonic::include_proto!("mdm.bp.v1");
    }
}
