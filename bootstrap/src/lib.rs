//! cuba-bootstrap - 统一服务启动骨架
//!
//! 所有服务复用的启动逻辑，包括：
//! - 配置加载
//! - 运行时初始化（日志、追踪）
//! - 基础设施资源管理（数据库、Redis、TokenService）
//! - gRPC 拦截器
//! - Graceful shutdown
//!
//! # 使用方式
//!
//! ```ignore
//! use cuba_bootstrap::{run, Infrastructure};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     run("config", |infra: Infrastructure| async move {
//!         // 从 Infrastructure 获取资源
//!         let pool = infra.postgres_pool();
//!         let cache = infra.redis_cache();
//!         let token_service = infra.token_service();
//!
//!         // 组装服务
//!         let service = MyServiceImpl::new(pool, cache, token_service);
//!         MyServiceServer::new(service)
//!     }).await
//! }
//! ```

mod infrastructure;
mod interceptor;
mod runtime;
mod shutdown;
mod starter;

pub use infrastructure::Infrastructure;
pub use interceptor::*;
pub use runtime::*;
pub use shutdown::*;
pub use starter::*;
