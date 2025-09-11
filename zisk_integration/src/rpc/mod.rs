//! RPC Service Layer for ZisK-Zcash Validator
//! 
//! This module provides HTTP/gRPC interfaces for external clients to submit
//! transaction batches and receive validation results and STARK proofs.

pub mod http_server;
// pub mod grpc_server; // Temporarily disabled
pub mod handlers;
pub mod serialization;
pub mod types;

pub use handlers::*;
pub use types::*;
