//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::TransportServiceImpl;

pub mod proto {
    pub mod tp {
        tonic::include_proto!("sc.tp.v1");
    }
}
