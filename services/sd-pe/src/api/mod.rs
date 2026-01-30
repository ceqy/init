//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::PricingEngineServiceImpl;

pub mod proto {
    pub mod pe {
        tonic::include_proto!("sd.pe.v1");
    }
}
