//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::SysNotifyServiceImpl;

pub mod proto {
    pub mod msg {
        tonic::include_proto!("sys.msg.v1");
    }
    pub mod job {
        tonic::include_proto!("sys.job.v1");
    }
}
