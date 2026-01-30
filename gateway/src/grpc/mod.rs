//! gRPC 客户端

pub mod auth {
    tonic::include_proto!("iam.auth.v1");
}

pub mod user {
    tonic::include_proto!("iam.user.v1");
}

pub mod audit {
    tonic::include_proto!("iam.audit.v1");
}

use audit::audit_service_client::AuditServiceClient;
use auth::auth_service_client::AuthServiceClient;
use std::time::Duration;
use tonic::transport::Channel;
use user::user_service_client::UserServiceClient;

/// gRPC 客户端集合
#[derive(Clone)]
pub struct GrpcClients {
    pub auth: AuthServiceClient<Channel>,
    pub user: UserServiceClient<Channel>,
    pub audit: AuditServiceClient<Channel>,
}

impl GrpcClients {
    /// 创建新的 gRPC 客户端集合
    ///
    /// 配置了适当的超时和连接设置以支持高并发请求
    pub async fn new(iam_endpoint: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(iam_endpoint)?
            // 连接超时
            .connect_timeout(Duration::from_secs(10))
            // 请求超时
            .timeout(Duration::from_secs(30))
            // 保持连接活跃
            .keep_alive_timeout(Duration::from_secs(20))
            .keep_alive_while_idle(true)
            // HTTP/2 并发流限制
            .http2_adaptive_window(true)
            // 初始连接窗口大小
            .initial_connection_window_size(1024 * 1024) // 1MB
            .initial_stream_window_size(1024 * 1024) // 1MB
            // 并发请求限制
            .concurrency_limit(1000)
            .connect()
            .await?;

        Ok(Self {
            auth: AuthServiceClient::new(channel.clone()),
            user: UserServiceClient::new(channel.clone()),
            audit: AuditServiceClient::new(channel),
        })
    }
}
