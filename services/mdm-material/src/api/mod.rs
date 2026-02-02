//! API layer - gRPC service implementations

pub mod conversions;
pub mod grpc_service;
pub mod proto_converters;

pub use grpc_service::MaterialServiceImpl;
