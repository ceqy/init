//! 用户 gRPC 服务

mod user_service_impl;

pub mod proto {
    tonic::include_proto!("cuba.iam.user");
    
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        include_bytes!("user_descriptor.bin");
}

pub use user_service_impl::*;
pub use proto::user_service_server::{UserService, UserServiceServer};
