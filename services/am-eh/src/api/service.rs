//! gRPC service implementation stub

use std::sync::Arc;
use crate::application::ServiceHandler;

pub struct EquipmentHealthServiceImpl {
    #[allow(dead_code)]
    handler: Arc<ServiceHandler>,
}

impl EquipmentHealthServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

// TODO: Implement gRPC service trait for EquipmentHealthService
