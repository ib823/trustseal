#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unused_imports)]
// Allow certain pedantic lints that conflict with our patterns
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]

//! Sahi Crypto Engine — KMS abstraction, SD-JWT, DID resolution, key management
//!
//! # Modules
//!
//! - `kms` — Key Management Service abstraction (F1). Trait + SoftwareKmsProvider.
//! - `merkle` — Merkle tree for tamper-evident logging (F2).
//! - `trust_registry` — Trust Registry data model (F6).
//! - `sd_jwt` — SD-JWT credential operations (VP-1).
//! - `did` — DID resolution (VP-2). did:key, did:web, caching.
//! - `status_list` — Bitstring Status List for revocation (VP-3).
//! - `credential_types` — VaultPass credential type definitions (VP-3).
//! - `compliance` — Compliance enforcement for credential issuance (VP-3b).
//! - `access_rules` — Business logic rules engine for access decisions (VP-3c).
//! - `bbs_plus` — BBS+ signatures for unlinkable presentations (Phase 2 stub).
//! - `error` — Typed error types with SAHI_XXXX codes.

pub mod access_rules;
pub mod bbs_plus;
pub mod compliance;
pub mod credential_types;
pub mod did;
pub mod error;
pub mod kms;
pub mod merkle;
pub mod sd_jwt;
pub mod status_list;
pub mod trust_registry;
