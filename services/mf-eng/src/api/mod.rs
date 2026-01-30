//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::EngineeringServiceImpl;

pub mod proto {
    pub mod eng {
        tonic::include_proto!("mf.eng.v1");
    }
}
