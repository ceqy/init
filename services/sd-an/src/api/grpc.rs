//! gRPC utilities and interceptors

use tonic::{Request, Status};

/// Tracing interceptor for gRPC requests
pub fn tracing_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    tracing::debug!("gRPC request: {:?}", req.metadata());
    Ok(req)
}
