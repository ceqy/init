//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ShopFloorServiceImpl;

pub mod proto {
    pub mod sf {
        tonic::include_proto!("mf.sf.v1");
    }
}
