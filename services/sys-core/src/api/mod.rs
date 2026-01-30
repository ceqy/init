//! API layer - gRPC service implementations

pub mod grpc;
mod service;

pub use service::SysCoreServiceImpl;

pub mod proto {
    pub mod nr {
        tonic::include_proto!("sys.nr.v1");
    }
    pub mod cfg {
        tonic::include_proto!("sys.cfg.v1");
    }
}
