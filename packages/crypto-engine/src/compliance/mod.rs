//! Compliance Enforcement Layer (VP-3b)
//!
//! Rules engine evaluating compliance before credential issuance.
//!
//! Per MASTER_PLAN section 8.4:
//! - Resident identity verified (eKYC via MyDigital ID)
//! - Unit ownership/tenancy confirmed by property admin
//! - No outstanding violations or blacklist entries
//! - Credential limits per resident not exceeded

mod checks;
mod engine;
mod types;

pub use checks::{
    BlacklistCheck, CredentialLimitCheck, IdentityVerificationCheck, UnitOwnershipCheck,
};
pub use engine::{ComplianceEngine, ComplianceResult};
pub use types::{
    BlacklistEntry, BlacklistReason, ComplianceCheck, ComplianceContext, ComplianceError,
    ComplianceStatus, IdentityVerification, UnitOwnership,
};
