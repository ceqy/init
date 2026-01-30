//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::ExpenseServiceImpl;

pub mod proto {
    pub mod ex {
        tonic::include_proto!("hr.ex.v1");
    }
}
