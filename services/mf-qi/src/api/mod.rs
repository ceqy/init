//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::QualityInspectionServiceImpl;

pub mod proto {
    pub mod qi {
        tonic::include_proto!("mf.qi.v1");
    }
}
