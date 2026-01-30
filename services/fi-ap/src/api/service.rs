//! gRPC service implementation stub

use std::sync::Arc;
use crate::application::ServiceHandler;

pub struct AccountsPayableServiceImpl {
    #[allow(dead_code)]
    handler: Arc<ServiceHandler>,
}

impl AccountsPayableServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

// TODO: Implement gRPC service trait for AccountsPayableService
