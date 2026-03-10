//! Status List implementation for credential revocation.
//!
//! Implements W3C Bitstring Status List v1.0:
//! <https://www.w3.org/TR/vc-bitstring-status-list/>
//!
//! # Architecture
//!
//! - **Bitstring**: Core compressed bitstring (gzip + base64)
//! - **Credential**: Status List Credential and entry types
//! - **Allocator**: Randomized index allocation for privacy
//! - **Manager**: Combines allocation, revocation, and credential generation
//!
//! # Usage
//!
//! ```ignore
//! use crypto_engine::status_list::{StatusListManager, StatusPurpose};
//!
//! // Create a manager for a tenant's credential type
//! let manager = StatusListManager::new(
//!     "TNT_01HXK",
//!     "ResidentBadge",
//!     "https://sahi.my",
//!     "did:web:status.sahi.my",
//! );
//!
//! // Allocate a status entry for a new credential
//! let entry = manager.allocate_revocation().unwrap();
//!
//! // Later, revoke the credential
//! let index = entry.parse_index().unwrap();
//! manager.revoke(index).unwrap();
//!
//! // Generate the Status List Credential for publication
//! let credential = manager.generate_credential().unwrap();
//! ```

mod allocator;
mod bitstring;
mod credential;
mod manager;

pub use allocator::IndexAllocator;
pub use bitstring::Bitstring;
pub use credential::{
    BitstringStatusListEntry, StatusListCredential, StatusListSubject, StatusPurpose,
};
pub use manager::{ExportedState, StatusListManager, StatusListRegistry, StatusListStats};
