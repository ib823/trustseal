#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unused_imports)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]

//! Sahi Core — shared types, error codes, ULID factory, time utilities
//!
//! # Modules
//!
//! - `error` — Platform-wide error code registry (Appendix E)
//! - `ulid` — ULID factory with typed prefixes (Appendix G)
//! - `time` — RFC 3339 time utilities

pub mod error;
pub mod time;
pub mod ulid;
