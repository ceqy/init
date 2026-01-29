//! gRPC 追踪拦截器
//!
//! 负责从请求元数据中提取追踪 ID (Trace ID / Correlation ID)
//! 并将其注入到日志上下文中。

use tonic::{Request, Status};
use tracing::info_span;
use uuid::Uuid;

/// 追踪信息
#[derive(Debug, Clone)]
pub struct TraceInfo {
    pub trace_id: String,
}

/// 用户信息 (从元数据提取)
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
}

/// gRPC 拦截器：提取追踪 ID 和用户信息
#[allow(clippy::result_large_err)]
pub fn tracing_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let metadata = req.metadata();

    // 优先从元数据提取 trace_id
    let trace_id = metadata
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        // 备选：x-request-id
        .or_else(|| {
            metadata
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        // 备选：x-correlation-id
        .or_else(|| {
            metadata
                .get("x-correlation-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        // 都没有则生成一个
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    // 提取用户信息
    let user_id = metadata
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "system".to_string());

    // 将信息注入到请求扩展中
    let mut req = req;
    req.extensions_mut().insert(TraceInfo { trace_id });
    req.extensions_mut().insert(UserInfo { user_id });

    Ok(req)
}

/// 辅助宏：或者助手函数来创建 span
pub fn create_request_span(
    req: &Request<impl std::fmt::Debug>,
    name: &'static str,
) -> tracing::Span {
    let trace_id = req
        .extensions()
        .get::<TraceInfo>()
        .map(|t| t.trace_id.as_str())
        .unwrap_or("unknown");

    info_span!(
        "grpc_request",
        span_name = name,
        trace_id = %trace_id
    )
}

/*
pub fn inject_trace_id<T>(req: &mut tonic::Request<T>) {
    let trace_id = tracing::Span::current()
        .field("trace_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            // 如果 Span 中没有，尝试从本地 ThreadLocal 或其他地方获取
            // 这里简单处理，如果当前没有活跃 Span，就不注入
            String::new()
        });

    if !trace_id.is_empty() {
        if let Ok(value) = trace_id.parse() {
            req.metadata_mut().insert("x-trace-id", value);
        }
    }
}
*/
