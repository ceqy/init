//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ChartOfAccountsServiceImpl;

pub mod proto {
    pub mod coa {
        tonic::include_proto!("fi.coa.v1");
    }
}
