//! flutter_rust_bridge API module.
//!
//! Exposes crypto-engine functionality to Dart via FFI.
//! All cryptographic operations happen in Rust - ZERO Dart crypto.

pub mod crypto;
pub mod ble_payload;
pub mod revocation;
