//! gRPC service implementation stub

use std::sync::Arc;
use crate::application::ServiceHandler;

pub struct EnterpriseServiceImpl {
    #[allow(dead_code)]
    handler: Arc<ServiceHandler>,
}

impl EnterpriseServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

// TODO: Implement gRPC service trait for EnterpriseService
