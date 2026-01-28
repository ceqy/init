//! gRPC 服务实现

pub mod auth_service;
pub mod audit_service;
pub mod oauth_service;
pub mod user_service;

// Proto 生成的代码模块
pub mod auth_proto {
    include!("proto_gen/cuba.iam.auth.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("proto_gen/auth_descriptor.bin");
}

pub mod user_proto {
    include!("proto_gen/cuba.iam.user.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("proto_gen/user_descriptor.bin");
}

pub mod oauth_proto {
    include!("proto_gen/cuba.iam.oauth.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("proto_gen/oauth_descriptor.bin");
}

pub mod audit_proto {
    include!("proto_gen/cuba.iam.audit.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("proto_gen/audit_descriptor.bin");
}
