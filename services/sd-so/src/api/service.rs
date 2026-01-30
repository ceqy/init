//! gRPC service implementation stub

use std::sync::Arc;
use crate::application::ServiceHandler;

pub struct SalesOrderServiceImpl {
    #[allow(dead_code)]
    handler: Arc<ServiceHandler>,
}

impl SalesOrderServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

// TODO: Implement gRPC service trait for SalesOrderService
