//! TM-1: Signing Orchestrator Domain Types
//!
//! Core domain types for the `TrustMark` signing ceremony state machine.

mod ceremony;
mod signer;
mod state_machine;

#[allow(unused_imports)]
pub use ceremony::{
    Ceremony, CeremonyConfig, CeremonyDocument, CeremonyId, CeremonyMetadata, CeremonyType,
    SignatureField,
};
#[allow(unused_imports)]
pub use signer::{
    SignatureData, SignerInvitation, SignerRole, SignerSlot, SignerSlotId, SignerStatus,
};
pub use state_machine::{CeremonyState, CeremonyTransition, StateTransitionError};
