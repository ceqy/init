//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::GeospatialServiceImpl;

pub mod proto {
    pub mod gs {
        tonic::include_proto!("am.gs.v1");
    }
}
