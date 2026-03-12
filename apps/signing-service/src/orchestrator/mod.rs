//! TM-1: Signing Ceremony Orchestrator
//!
//! The orchestrator manages ceremony lifecycle, state transitions,
//! and coordinates between signers, documents, and the Merkle log.

mod error;
mod service;

#[allow(unused_imports)]
pub use error::OrchestratorError;
pub use service::CeremonyOrchestrator;
