//! gRPC 客户端

pub mod auth {
    tonic::include_proto!("cuba.iam.auth");
}

pub mod user {
    tonic::include_proto!("cuba.iam.user");
}

use auth::auth_service_client::AuthServiceClient;
use user::user_service_client::UserServiceClient;
use tonic::transport::Channel;

/// gRPC 客户端集合
#[derive(Clone)]
pub struct GrpcClients {
    pub auth: AuthServiceClient<Channel>,
    pub user: UserServiceClient<Channel>,
}

impl GrpcClients {
    /// 创建新的 gRPC 客户端集合
    pub async fn new(iam_endpoint: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(iam_endpoint)?
            .connect()
            .await?;

        Ok(Self {
            auth: AuthServiceClient::new(channel.clone()),
            user: UserServiceClient::new(channel),
        })
    }
}
