//! VaultPass Wallet Rust library.
//!
//! Provides cryptographic operations via flutter_rust_bridge FFI.
//! All crypto happens in Rust - ZERO Dart crypto per spec.

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // FFI functions - errors documented in Dart layer
#![allow(clippy::uninlined_format_args)] // Clearer for FFI error messages
#![allow(clippy::cast_possible_truncation)] // Bounds checked before cast
#![allow(clippy::doc_markdown)] // FFI crate - doc comments are minimal

pub mod api;

// Re-export API functions for flutter_rust_bridge
pub use api::crypto::*;
pub use api::ble_payload::*;
pub use api::revocation::*;
