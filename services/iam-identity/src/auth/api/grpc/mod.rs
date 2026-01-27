//! 认证 gRPC 服务

mod auth_service_impl;

pub mod proto {
    include!("cuba.iam.auth.rs");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("auth_descriptor.bin");
}

pub use auth_service_impl::*;
pub use proto::auth_service_server::{AuthService, AuthServiceServer};
