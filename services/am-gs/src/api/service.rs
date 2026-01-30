//! gRPC service implementation stub

use std::sync::Arc;
use crate::application::ServiceHandler;

pub struct GeospatialServiceImpl {
    #[allow(dead_code)]
    handler: Arc<ServiceHandler>,
}

impl GeospatialServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

// TODO: Implement gRPC service trait for GeospatialService
