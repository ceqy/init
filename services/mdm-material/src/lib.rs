//! Service library

pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;

// Proto generated code modules
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod mdm_material {
    pub mod v1 {
        tonic::include_proto!("mdm.material.v1");
    }
}

// Re-export for convenience
pub use mdm_material::v1 as proto;
