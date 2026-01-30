//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::WarehouseServiceImpl;

pub mod proto {
    pub mod wm {
        tonic::include_proto!("sc.wm.v1");
    }
}
