pub mod oauth_service_impl;

pub mod proto {
    include!("cuba.iam.oauth.rs");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("oauth_descriptor.bin");
}

pub use oauth_service_impl::OAuthServiceImpl;
pub use proto::o_auth_service_server::{OAuthService, OAuthServiceServer};
