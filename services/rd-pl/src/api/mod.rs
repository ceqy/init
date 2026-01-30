//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ProductLifecycleServiceImpl;

pub mod proto {
    pub mod pl {
        tonic::include_proto!("rd.pl.v1");
    }
}
