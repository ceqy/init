//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::VendorServiceImpl;

pub mod proto {
    pub mod vs {
        tonic::include_proto!("pm.vs.v1");
    }
}
