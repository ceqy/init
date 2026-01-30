//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::TimeAttendanceServiceImpl;

pub mod proto {
    pub mod ta {
        tonic::include_proto!("hr.ta.v1");
    }
}
