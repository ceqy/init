//! Business logic handler

use std::sync::Arc;

use crate::domain::repositories::MaterialRepository;

pub struct ServiceHandler {
    #[allow(dead_code)]
    material_repo: Arc<dyn MaterialRepository>,
}

impl ServiceHandler {
    pub fn new(material_repo: Arc<dyn MaterialRepository>) -> Self {
        Self { material_repo }
    }
}

// TODO: Implement handler methods
