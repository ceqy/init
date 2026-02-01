//! Persistence implementations

mod postgres;

pub use postgres::{PostgresMaterialGroupRepository, PostgresMaterialRepository, PostgresMaterialTypeRepository};
