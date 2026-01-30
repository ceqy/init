//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::InventoryServiceImpl;

pub mod proto {
    pub mod im {
        tonic::include_proto!("sc.im.v1");
    }
}
