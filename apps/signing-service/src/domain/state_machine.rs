//! TM-1: Signing Ceremony State Machine
//!
//! Implements the ceremony state machine per `MASTER_PLAN` Section 12.1:
//!
//! ```text
//! CREATED -> PREPARING -> READY_FOR_SIGNATURES -> SIGNING_IN_PROGRESS
//!     -> PARTIALLY_SIGNED -> FULLY_SIGNED -> TIMESTAMPING
//!     -> AUGMENTING_LTV -> COMPLETE
//!
//! (Any state) -> ABORTED
//! ABORTED -> RESUMING -> (previous state, with document hash verification)
//! ```

// Allow pass-by-ref for small Copy types - this is a stub module
#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Ceremony state following the `TrustMark` signing flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CeremonyState {
    /// Initial state after ceremony creation
    Created,
    /// Document processing and signer invitation in progress
    Preparing,
    /// All prerequisites met, ready for signers
    ReadyForSignatures,
    /// At least one signer is actively signing
    SigningInProgress,
    /// Some signers have completed, others pending
    PartiallySigned,
    /// All required signatures collected
    FullySigned,
    /// Adding RFC 3161 timestamp to signatures
    Timestamping,
    /// Fetching OCSP/CRL for LTV (Long-Term Validation)
    AugmentingLtv,
    /// All processing complete, document finalized
    Complete,
    /// Ceremony cancelled or failed
    Aborted,
    /// Resuming from aborted state (requires document hash verification)
    Resuming,
}

impl CeremonyState {
    /// Returns true if this is a terminal state (no further transitions allowed).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Aborted)
    }

    /// Returns true if signers can submit signatures in this state.
    #[must_use]
    pub const fn accepts_signatures(&self) -> bool {
        matches!(
            self,
            Self::ReadyForSignatures | Self::SigningInProgress | Self::PartiallySigned
        )
    }

    /// Returns true if the ceremony can be aborted from this state.
    #[must_use]
    pub const fn can_abort(&self) -> bool {
        !self.is_terminal()
    }

    /// Returns the valid next states from this state.
    #[must_use]
    pub fn valid_transitions(&self) -> &'static [CeremonyState] {
        match self {
            Self::Created => &[Self::Preparing, Self::Aborted],
            Self::Preparing => &[Self::ReadyForSignatures, Self::Aborted],
            Self::ReadyForSignatures => &[Self::SigningInProgress, Self::Aborted],
            Self::SigningInProgress => &[Self::PartiallySigned, Self::FullySigned, Self::Aborted],
            Self::PartiallySigned => &[Self::SigningInProgress, Self::FullySigned, Self::Aborted],
            Self::FullySigned => &[Self::Timestamping, Self::Aborted],
            Self::Timestamping => &[Self::AugmentingLtv, Self::Aborted],
            Self::AugmentingLtv => &[Self::Complete, Self::Aborted],
            Self::Complete => &[],
            Self::Aborted => &[Self::Resuming],
            Self::Resuming => &[
                Self::Created,
                Self::Preparing,
                Self::ReadyForSignatures,
                Self::SigningInProgress,
                Self::PartiallySigned,
                Self::FullySigned,
                Self::Aborted,
            ],
        }
    }

    /// Check if a transition to the target state is valid.
    #[must_use]
    pub fn can_transition_to(&self, target: &Self) -> bool {
        self.valid_transitions().contains(target)
    }

    /// Database representation
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Preparing => "PREPARING",
            Self::ReadyForSignatures => "READY_FOR_SIGNATURES",
            Self::SigningInProgress => "SIGNING_IN_PROGRESS",
            Self::PartiallySigned => "PARTIALLY_SIGNED",
            Self::FullySigned => "FULLY_SIGNED",
            Self::Timestamping => "TIMESTAMPING",
            Self::AugmentingLtv => "AUGMENTING_LTV",
            Self::Complete => "COMPLETE",
            Self::Aborted => "ABORTED",
            Self::Resuming => "RESUMING",
        }
    }

    /// Parse from database string.
    ///
    /// Returns `Created` for unknown values (database constraints should prevent this).
    #[must_use]
    #[allow(clippy::match_same_arms)] // Wildcard fallback is intentional for safety
    pub fn from_str(s: &str) -> Self {
        match s {
            "CREATED" => Self::Created,
            "PREPARING" => Self::Preparing,
            "READY_FOR_SIGNATURES" => Self::ReadyForSignatures,
            "SIGNING_IN_PROGRESS" => Self::SigningInProgress,
            "PARTIALLY_SIGNED" => Self::PartiallySigned,
            "FULLY_SIGNED" => Self::FullySigned,
            "TIMESTAMPING" => Self::Timestamping,
            "AUGMENTING_LTV" => Self::AugmentingLtv,
            "COMPLETE" => Self::Complete,
            "ABORTED" => Self::Aborted,
            "RESUMING" => Self::Resuming,
            _ => Self::Created, // DB constraints prevent this
        }
    }
}

impl fmt::Display for CeremonyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// State transition error.
#[derive(Debug, Error)]
pub enum StateTransitionError {
    /// Invalid state transition attempted
    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition {
        from: CeremonyState,
        to: CeremonyState,
    },

    /// Ceremony is in a terminal state
    #[error("Ceremony is in terminal state {0} and cannot transition")]
    TerminalState(CeremonyState),

    /// Precondition for transition not met
    #[error("Precondition not met: {0}")]
    PreconditionNotMet(String),

    /// Optimistic locking conflict
    #[error("Version conflict: expected {expected}, found {found}")]
    VersionConflict { expected: i64, found: i64 },
}

/// Represents a state transition event to be logged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyTransition {
    /// The state before transition
    pub from_state: CeremonyState,
    /// The state after transition
    pub to_state: CeremonyState,
    /// Reason for the transition
    pub reason: String,
    /// Actor who triggered the transition (user ID or system)
    pub actor: String,
    /// Timestamp of the transition (RFC 3339)
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_created_transitions() {
        let state = CeremonyState::Created;
        assert!(state.can_transition_to(&CeremonyState::Preparing));
        assert!(state.can_transition_to(&CeremonyState::Aborted));
        assert!(!state.can_transition_to(&CeremonyState::Complete));
        assert!(!state.can_transition_to(&CeremonyState::SigningInProgress));
    }

    #[test]
    fn test_happy_path_transitions() {
        // Verify the full happy path is valid
        let happy_path = [
            CeremonyState::Created,
            CeremonyState::Preparing,
            CeremonyState::ReadyForSignatures,
            CeremonyState::SigningInProgress,
            CeremonyState::PartiallySigned,
            CeremonyState::FullySigned,
            CeremonyState::Timestamping,
            CeremonyState::AugmentingLtv,
            CeremonyState::Complete,
        ];

        for window in happy_path.windows(2) {
            let from = &window[0];
            let to = &window[1];
            assert!(
                from.can_transition_to(to),
                "Should be able to transition from {from} to {to}"
            );
        }
    }

    #[test]
    fn test_abort_from_any_non_terminal() {
        let non_terminal = [
            CeremonyState::Created,
            CeremonyState::Preparing,
            CeremonyState::ReadyForSignatures,
            CeremonyState::SigningInProgress,
            CeremonyState::PartiallySigned,
            CeremonyState::FullySigned,
            CeremonyState::Timestamping,
            CeremonyState::AugmentingLtv,
            CeremonyState::Resuming,
        ];

        for state in &non_terminal {
            assert!(state.can_abort(), "{state} should be abortable");
            assert!(
                state.can_transition_to(&CeremonyState::Aborted),
                "{state} should transition to Aborted"
            );
        }
    }

    #[test]
    fn test_terminal_states_no_transitions() {
        assert!(CeremonyState::Complete.is_terminal());
        assert!(CeremonyState::Complete.valid_transitions().is_empty());
        assert!(!CeremonyState::Complete.can_abort());
    }

    #[test]
    fn test_aborted_can_resume() {
        let aborted = CeremonyState::Aborted;
        assert!(aborted.can_transition_to(&CeremonyState::Resuming));
        assert!(!aborted.can_abort()); // Already aborted
    }

    #[test]
    fn test_resuming_can_return_to_previous_states() {
        let resuming = CeremonyState::Resuming;
        assert!(resuming.can_transition_to(&CeremonyState::Created));
        assert!(resuming.can_transition_to(&CeremonyState::Preparing));
        assert!(resuming.can_transition_to(&CeremonyState::ReadyForSignatures));
        assert!(resuming.can_transition_to(&CeremonyState::SigningInProgress));
        // But not directly to Complete (must go through the flow)
        assert!(!resuming.can_transition_to(&CeremonyState::Complete));
    }

    #[test]
    fn test_accepts_signatures() {
        assert!(CeremonyState::ReadyForSignatures.accepts_signatures());
        assert!(CeremonyState::SigningInProgress.accepts_signatures());
        assert!(CeremonyState::PartiallySigned.accepts_signatures());
        assert!(!CeremonyState::Created.accepts_signatures());
        assert!(!CeremonyState::FullySigned.accepts_signatures());
        assert!(!CeremonyState::Complete.accepts_signatures());
    }

    #[test]
    fn test_state_serialization() {
        let state = CeremonyState::ReadyForSignatures;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"READY_FOR_SIGNATURES\"");

        let parsed: CeremonyState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_state_from_str() {
        assert_eq!(CeremonyState::from_str("CREATED"), CeremonyState::Created);
        assert_eq!(
            CeremonyState::from_str("AUGMENTING_LTV"),
            CeremonyState::AugmentingLtv
        );
        // Unknown values default to Created (DB constraints should prevent this)
        assert_eq!(CeremonyState::from_str("INVALID"), CeremonyState::Created);
    }

    #[test]
    fn test_state_display() {
        assert_eq!(
            CeremonyState::SigningInProgress.to_string(),
            "SIGNING_IN_PROGRESS"
        );
        assert_eq!(CeremonyState::AugmentingLtv.to_string(), "AUGMENTING_LTV");
    }
}
