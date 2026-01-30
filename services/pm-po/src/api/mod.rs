//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::PurchaseOrderServiceImpl;

pub mod proto {
    pub mod po {
        tonic::include_proto!("pm.po.v1");
    }
}
