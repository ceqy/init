//! Service library

pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;

// Common types at crate level for proto generated code
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
