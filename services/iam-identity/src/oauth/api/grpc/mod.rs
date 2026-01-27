pub mod oauth_service_impl;

pub mod proto {
    tonic::include_proto!("cuba.iam.oauth");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("oauth_descriptor.bin");
}

pub use oauth_service_impl::OAuthServiceImpl;
pub use proto::o_auth_service_server::{OAuthService, OAuthServiceServer};
