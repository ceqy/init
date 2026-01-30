//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::GeneralLedgerServiceImpl;

pub mod proto {
    pub mod gl {
        tonic::include_proto!("fi.gl.v1");
    }
}
