//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::AccountsPayableServiceImpl;

pub mod proto {
    pub mod ap {
        tonic::include_proto!("fi.ap.v1");
    }
}
