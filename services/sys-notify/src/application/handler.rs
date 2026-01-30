//! Business logic handler

use std::sync::Arc;
use crate::domain::Repository;

pub struct ServiceHandler {
    #[allow(dead_code)]
    repo: Arc<dyn Repository>,
}

impl ServiceHandler {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

// TODO: Implement handler methods for Message and Job
