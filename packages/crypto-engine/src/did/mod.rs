//! DID resolution for VaultPass (VP-2).
//!
//! This module provides DID resolution across multiple methods:
//!
//! - `did:key` — Pure cryptographic DIDs (no network, derive from public key)
//! - `did:web` — HTTP-based DIDs (fetch from `.well-known/did.json`)
//! - `did:peer` — Pairwise unlinkable DIDs (Phase 2 stub)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    DidResolver                          │
//! │  ┌────────┐  ┌────────┐  ┌────────┐                    │
//! │  │did:key │  │did:web │  │did:peer│                    │
//! │  │resolver│  │resolver│  │ (stub) │                    │
//! │  └───┬────┘  └───┬────┘  └────────┘                    │
//! │      │           │                                      │
//! │      └─────┬─────┘                                      │
//! │            ▼                                            │
//! │     ┌──────────┐                                        │
//! │     │DidCache  │  L1: In-memory LRU                    │
//! │     │          │  L2: Redis (optional)                  │
//! │     └──────────┘  L3: PostgreSQL (audit)               │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # DID Methods by Entity
//!
//! | Entity | Method | Example |
//! |--------|--------|---------|
//! | Platform | `did:web` | `did:web:sahi.my` |
//! | Property/Tenant | `did:web` | `did:web:property.sahi.my` |
//! | Resident (Phase 1) | `did:key` | `did:key:z6Mkh...` |
//! | Resident (Phase 2) | `did:peer` | Pairwise, unlinkable |
//! | Edge Verifier | `did:web` | `did:web:gate-a.property.sahi.my` |
//!
//! # Usage
//!
//! ```ignore
//! use crypto_engine::did::{DidResolver, ResolverConfig, VerificationPurpose};
//!
//! // Create resolver
//! let resolver = DidResolver::default();
//!
//! // Resolve did:key (synchronous, no network)
//! let result = resolver.resolve("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")?;
//! let doc = result.document.unwrap();
//!
//! // Get public key for signature verification
//! let key = resolver.resolve_issuer_key("did:key:z6Mkh...")?;
//! ```
//!
//! # Performance
//!
//! - `did:key` resolution: < 1ms (pure computation)
//! - `did:web` resolution (uncached): depends on network
//! - `did:web` resolution (cached): < 1ms
//!
//! Cache TTLs (configurable):
//! - `did:key`: 1 hour (deterministic, no need to refresh often)
//! - `did:web`: 1 hour (balance freshness vs. load)

mod cache;
mod did_key;
mod did_web;
mod resolver;
mod types;

// Re-export main types
pub use cache::{redis_key, CacheStats, CachedDocument, DidCache, REDIS_PREFIX};
pub use did_key::{from_ed25519_public_key, from_p256_public_key, KeyType};
pub use did_web::{did_to_url, validate_document, DidWebConfig};
pub use resolver::{AsyncDidResolver, DidResolver, ResolverConfig};
pub use types::{
    ControllerSet, Did, DidComponents, DidContext, DidDocument, ResolutionMetadata,
    ResolutionResult, ServiceEndpoint, ServiceEndpointValue, VerificationMethod,
    VerificationPurpose, VerificationRelationship,
};

/// Resolve a did:key directly (convenience function).
///
/// # Errors
/// Returns an error if the DID is invalid.
pub fn resolve_did_key(did: &str) -> Result<DidDocument, String> {
    did_key::resolve(did)
}
