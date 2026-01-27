//! gRPC 服务实现

pub mod auth_service;
pub mod oauth_service;
pub mod user_service;

// Proto 模块保留在原位置
pub mod auth_proto {
    include!("../../auth/api/grpc/cuba.iam.auth.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../auth/api/grpc/auth_descriptor.bin");
}

pub mod user_proto {
    include!("../../user/api/grpc/cuba.iam.user.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../user/api/grpc/user_descriptor.bin");
}

pub mod oauth_proto {
    include!("../../oauth/api/grpc/cuba.iam.oauth.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../oauth/api/grpc/oauth_descriptor.bin");
}
