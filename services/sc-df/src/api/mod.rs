//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::DemandForecastServiceImpl;

pub mod proto {
    pub mod df {
        tonic::include_proto!("sc.df.v1");
    }
}
