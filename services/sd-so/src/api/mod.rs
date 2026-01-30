//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::SalesOrderServiceImpl;

pub mod proto {
    pub mod so {
        tonic::include_proto!("sd.so.v1");
    }
}
