//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ProductionPlanningServiceImpl;

pub mod proto {
    pub mod pp {
        tonic::include_proto!("mf.pp.v1");
    }
}
