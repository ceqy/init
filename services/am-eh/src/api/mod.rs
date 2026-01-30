//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::EquipmentHealthServiceImpl;

pub mod proto {
    pub mod eh {
        tonic::include_proto!("am.eh.v1");
    }
}
