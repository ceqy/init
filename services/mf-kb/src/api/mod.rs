//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::KanbanServiceImpl;

pub mod proto {
    pub mod kb {
        tonic::include_proto!("mf.kb.v1");
    }
}
