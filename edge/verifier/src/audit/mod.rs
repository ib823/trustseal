//! Append-only audit log for access decisions.
//!
//! Stores all access decisions locally for later sync to platform.
//! Uses SQLite with append-only semantics (no updates/deletes).

mod log;

pub use log::AuditLog;
