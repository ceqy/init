//! gRPC service implementation stub

use std::sync::Arc;
use crate::application::ServiceHandler;

pub struct WorkOrderServiceImpl {
    #[allow(dead_code)]
    handler: Arc<ServiceHandler>,
}

impl WorkOrderServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

// TODO: Implement gRPC service trait for WorkOrderService
