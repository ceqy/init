//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::AccountsReceivableServiceImpl;

pub mod proto {
    pub mod ar {
        tonic::include_proto!("fi.ar.v1");
    }
}
