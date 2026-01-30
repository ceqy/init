//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::PreventiveMaintenanceServiceImpl;

pub mod proto {
    pub mod pm {
        tonic::include_proto!("am.pm.v1");
    }
}
