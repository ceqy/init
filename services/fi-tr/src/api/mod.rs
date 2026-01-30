//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::TreasuryServiceImpl;

pub mod proto {
    pub mod tr {
        tonic::include_proto!("fi.tr.v1");
    }
}
