//! gRPC utilities and interceptors

use tonic::{Request, Status};

pub fn tracing_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    tracing::debug!("gRPC request: {:?}", req.metadata());
    Ok(req)
}
