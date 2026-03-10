#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unused_imports)]
// Allow certain pedantic lints that conflict with our patterns
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]

//! Sahi Crypto Engine — KMS abstraction, SD-JWT, key management, WASM bindings
//!
//! # Modules
//!
//! - `kms` — Key Management Service abstraction (F1). Trait + SoftwareKmsProvider.
//! - `error` — Typed error types with SAHI_XXXX codes.

pub mod error;
pub mod kms;
pub mod merkle;
pub mod trust_registry;
