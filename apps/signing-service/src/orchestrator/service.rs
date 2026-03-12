//! TM-1: Ceremony Orchestrator Service
//!
//! Manages ceremony lifecycle and state transitions.

// Allow unused_async - these methods are stubs that will have database operations
#![allow(clippy::unused_async)]

use super::error::OrchestratorError;
use crate::domain::{
    Ceremony, CeremonyConfig, CeremonyDocument, CeremonyId, CeremonyMetadata, CeremonyState,
    CeremonyTransition, CeremonyType, SignerSlot, SignerSlotId, SignerStatus,
};
use sahi_core::ulid::{generate, UlidPrefix};

/// Ceremony orchestrator service.
///
/// This service manages the lifecycle of signing ceremonies:
/// - Creation and configuration
/// - State transitions with validation
/// - Signer management
/// - Merkle log integration (state transitions)
pub struct CeremonyOrchestrator {
    // Database pool (to be added)
    // KMS provider (to be added)
    // Merkle log service (to be added)
}

impl CeremonyOrchestrator {
    /// Create a new orchestrator.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Create a new signing ceremony.
    ///
    /// # Errors
    /// Returns error if database or Merkle log operations fail.
    pub async fn create_ceremony(
        &self,
        tenant_id: &str,
        created_by: &str,
        document: CeremonyDocument,
        signers: Vec<SignerSlot>,
        config: CeremonyConfig,
        metadata: CeremonyMetadata,
    ) -> Result<Ceremony, OrchestratorError> {
        // Generate ceremony ID
        if signers.is_empty() {
            return Err(OrchestratorError::DocumentError(
                "At least one signer is required".to_string(),
            ));
        }

        let id = CeremonyId(generate(UlidPrefix::Ceremony));

        // Calculate expiry
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::hours(i64::from(config.ttl_hours));

        let ceremony = Ceremony {
            id,
            tenant_id: tenant_id.to_string(),
            created_by: created_by.to_string(),
            state: CeremonyState::Created,
            state_before_abort: None,
            ceremony_type: classify_ceremony_type(&signers),
            document,
            signers,
            config,
            metadata,
            version: 1,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
        };

        // TODO: Persist to database
        // TODO: Log to Merkle tree

        Ok(ceremony)
    }

    /// Transition ceremony state.
    ///
    /// Validates the transition and logs to Merkle tree.
    ///
    /// # Errors
    /// Returns error if transition is invalid or logging fails.
    pub async fn transition_state(
        &self,
        ceremony: &mut Ceremony,
        target_state: CeremonyState,
        reason: &str,
        actor: &str,
    ) -> Result<CeremonyTransition, OrchestratorError> {
        // Check expiry
        let now = chrono::Utc::now().to_rfc3339();
        if ceremony.is_expired(&now) {
            return Err(OrchestratorError::CeremonyExpiredError);
        }

        // Validate transition
        if !ceremony.state.can_transition_to(&target_state) {
            return Err(OrchestratorError::InvalidTransition(
                crate::domain::StateTransitionError::InvalidTransition {
                    from: ceremony.state,
                    to: target_state,
                },
            ));
        }

        // Create transition record
        let transition = CeremonyTransition {
            from_state: ceremony.state,
            to_state: target_state,
            reason: reason.to_string(),
            actor: actor.to_string(),
            timestamp: now.clone(),
        };

        // Handle abort specially - save previous state
        if target_state == CeremonyState::Aborted {
            ceremony.state_before_abort = Some(ceremony.state);
        }

        // Update ceremony
        ceremony.state = target_state;
        ceremony.version += 1;
        ceremony.updated_at = now;

        // TODO: Persist to database
        // TODO: Log to Merkle tree

        Ok(transition)
    }

    /// Start ceremony preparation (upload documents, invite signers).
    pub async fn prepare_ceremony(
        &self,
        ceremony: &mut Ceremony,
        actor: &str,
    ) -> Result<CeremonyTransition, OrchestratorError> {
        self.transition_state(
            ceremony,
            CeremonyState::Preparing,
            "Starting ceremony preparation",
            actor,
        )
        .await
    }

    /// Mark ceremony ready for signatures.
    pub async fn ready_for_signatures(
        &self,
        ceremony: &mut Ceremony,
        actor: &str,
    ) -> Result<CeremonyTransition, OrchestratorError> {
        // Verify all signers have been invited
        let uninvited = ceremony
            .signers
            .iter()
            .filter(|s| s.status == SignerStatus::Pending)
            .count();

        if uninvited > 0 {
            return Err(OrchestratorError::SignersIncomplete { pending: uninvited });
        }

        self.transition_state(
            ceremony,
            CeremonyState::ReadyForSignatures,
            "All signers invited, ready for signatures",
            actor,
        )
        .await
    }

    /// Record a signer authentication via `WebAuthn`.
    pub async fn authenticate_signer(
        &self,
        ceremony: &mut Ceremony,
        signer_id: &SignerSlotId,
        webauthn_credential_id: &str,
        assurance_level: &str,
    ) -> Result<(), OrchestratorError> {
        let signer = ceremony
            .signers
            .iter_mut()
            .find(|s| &s.id == signer_id)
            .ok_or_else(|| OrchestratorError::SignerNotFound(signer_id.to_string()))?;

        // Validate assurance level
        if !assurance_meets_requirement(assurance_level, &ceremony.config.min_assurance_level) {
            return Err(OrchestratorError::AssuranceLevelTooLow {
                required: ceremony.config.min_assurance_level.clone(),
                actual: assurance_level.to_string(),
            });
        }

        // Validate state transition
        if !signer
            .status
            .can_transition_to(&SignerStatus::Authenticated)
        {
            return Err(OrchestratorError::SignerNotReady);
        }

        signer.status = SignerStatus::Authenticated;
        signer.webauthn_credential_id = Some(webauthn_credential_id.to_string());
        signer.assurance_level = Some(assurance_level.to_string());

        // Transition ceremony if this is the first authentication
        if ceremony.state == CeremonyState::ReadyForSignatures {
            ceremony.state = CeremonyState::SigningInProgress;
            ceremony.version += 1;
        }

        Ok(())
    }

    /// Record a signature from a signer.
    pub async fn record_signature(
        &self,
        ceremony: &mut Ceremony,
        signer_id: &SignerSlotId,
        signature_data: crate::domain::SignatureData,
    ) -> Result<(), OrchestratorError> {
        let now = chrono::Utc::now().to_rfc3339();

        // Check expiry
        if ceremony.is_expired(&now) {
            return Err(OrchestratorError::CeremonyExpiredError);
        }

        // Find signer
        let signer = ceremony
            .signers
            .iter_mut()
            .find(|s| &s.id == signer_id)
            .ok_or_else(|| OrchestratorError::SignerNotFound(signer_id.to_string()))?;

        // Validate signer can sign
        if signer.status != SignerStatus::Authenticated {
            return Err(OrchestratorError::SignerNotReady);
        }

        // Record signature
        signer.status = SignerStatus::Signed;
        signer.signed_at = Some(now.clone());
        signer.signature_data = Some(signature_data);

        // Update ceremony state
        if ceremony.all_required_signed() {
            ceremony.state = CeremonyState::FullySigned;
        } else if ceremony.signed_count() > 0 {
            ceremony.state = CeremonyState::PartiallySigned;
        }

        ceremony.version += 1;
        ceremony.updated_at = now;

        Ok(())
    }

    /// Abort a ceremony.
    pub async fn abort_ceremony(
        &self,
        ceremony: &mut Ceremony,
        reason: &str,
        actor: &str,
    ) -> Result<CeremonyTransition, OrchestratorError> {
        if !ceremony.state.can_abort() {
            return Err(OrchestratorError::AbortFailed(
                "Ceremony is in terminal state".to_string(),
            ));
        }

        self.transition_state(ceremony, CeremonyState::Aborted, reason, actor)
            .await
    }

    /// Resume an aborted ceremony.
    pub async fn resume_ceremony(
        &self,
        ceremony: &mut Ceremony,
        document_hash: &str,
        actor: &str,
    ) -> Result<CeremonyTransition, OrchestratorError> {
        if ceremony.state != CeremonyState::Aborted {
            return Err(OrchestratorError::ResumeFailed(
                "Ceremony is not aborted".to_string(),
            ));
        }

        // Verify document hash matches
        if ceremony.document.content_hash != document_hash {
            return Err(OrchestratorError::DocumentHashMismatch {
                expected: ceremony.document.content_hash.clone(),
                actual: document_hash.to_string(),
            });
        }

        // First transition to Resuming
        self.transition_state(
            ceremony,
            CeremonyState::Resuming,
            "Resuming ceremony after abort",
            actor,
        )
        .await?;

        // Then restore to previous state
        let target_state = ceremony
            .state_before_abort
            .ok_or_else(|| OrchestratorError::ResumeFailed("No previous state".to_string()))?;

        self.transition_state(ceremony, target_state, "Restored to previous state", actor)
            .await
    }
}

impl Default for CeremonyOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_ceremony_type(signers: &[SignerSlot]) -> CeremonyType {
    match signers.len() {
        0 | 1 => CeremonyType::SingleSigner,
        _ => {
            let first_order = signers[0].order;
            if signers.iter().all(|signer| signer.order == first_order) {
                CeremonyType::MultiSignerParallel
            } else {
                CeremonyType::MultiSignerSequential
            }
        }
    }
}

/// Check if an assurance level meets a requirement.
fn assurance_meets_requirement(actual: &str, required: &str) -> bool {
    let level_value = |level: &str| -> u8 {
        match level {
            "P1" => 1,
            "P2" => 2,
            "P3" => 3,
            _ => 0,
        }
    };

    level_value(actual) >= level_value(required)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SignerRole;

    fn make_test_document() -> CeremonyDocument {
        CeremonyDocument {
            id: "DOC_test".to_string(),
            filename: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            content_hash: "abc123".to_string(),
            size_bytes: 1024,
            storage_key: "documents/test.pdf".to_string(),
            signature_fields: vec![],
        }
    }

    fn make_test_signer(id: &str, order: u32) -> SignerSlot {
        SignerSlot {
            id: SignerSlotId(format!("SLT_{id}")),
            order,
            name: format!("Signer {order}"),
            email: format!("signer{order}@example.com"),
            role: SignerRole::Signatory,
            is_required: true,
            status: SignerStatus::Pending,
            invitation: None,
            webauthn_credential_id: None,
            assurance_level: None,
            signed_at: None,
            signature_data: None,
            decline_reason: None,
        }
    }

    #[tokio::test]
    async fn test_create_ceremony() {
        let orchestrator = CeremonyOrchestrator::new();

        let document = make_test_document();
        let signers = vec![make_test_signer("001", 1)];
        let config = CeremonyConfig::default();
        let metadata = CeremonyMetadata {
            title: "Test Ceremony".to_string(),
            description: None,
            reference: None,
            tags: vec![],
        };

        let ceremony = orchestrator
            .create_ceremony("TNT_test", "USR_test", document, signers, config, metadata)
            .await
            .unwrap();

        assert_eq!(ceremony.state, CeremonyState::Created);
        assert!(ceremony.id.as_str().starts_with("CER_"));
        assert_eq!(ceremony.signers.len(), 1);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let orchestrator = CeremonyOrchestrator::new();

        let document = make_test_document();
        let mut signer = make_test_signer("001", 1);
        signer.status = SignerStatus::Invited; // Pre-invite for test
        let signers = vec![signer];
        let config = CeremonyConfig::default();
        let metadata = CeremonyMetadata {
            title: "Test".to_string(),
            description: None,
            reference: None,
            tags: vec![],
        };

        let mut ceremony = orchestrator
            .create_ceremony("TNT_test", "USR_test", document, signers, config, metadata)
            .await
            .unwrap();

        // Created -> Preparing
        let transition = orchestrator
            .prepare_ceremony(&mut ceremony, "USR_test")
            .await
            .unwrap();

        assert_eq!(transition.from_state, CeremonyState::Created);
        assert_eq!(transition.to_state, CeremonyState::Preparing);
        assert_eq!(ceremony.state, CeremonyState::Preparing);

        // Preparing -> ReadyForSignatures
        let transition = orchestrator
            .ready_for_signatures(&mut ceremony, "USR_test")
            .await
            .unwrap();

        assert_eq!(transition.to_state, CeremonyState::ReadyForSignatures);
    }

    #[tokio::test]
    async fn test_abort_and_resume() {
        let orchestrator = CeremonyOrchestrator::new();

        let document = make_test_document();
        let signers = vec![make_test_signer("001", 1)];
        let config = CeremonyConfig::default();
        let metadata = CeremonyMetadata {
            title: "Test".to_string(),
            description: None,
            reference: None,
            tags: vec![],
        };

        let mut ceremony = orchestrator
            .create_ceremony("TNT_test", "USR_test", document, signers, config, metadata)
            .await
            .unwrap();

        // Abort
        orchestrator
            .abort_ceremony(&mut ceremony, "Test abort", "USR_test")
            .await
            .unwrap();

        assert_eq!(ceremony.state, CeremonyState::Aborted);
        assert_eq!(ceremony.state_before_abort, Some(CeremonyState::Created));

        // Resume
        orchestrator
            .resume_ceremony(&mut ceremony, "abc123", "USR_test")
            .await
            .unwrap();

        assert_eq!(ceremony.state, CeremonyState::Created);
    }

    #[test]
    fn test_assurance_level_comparison() {
        assert!(assurance_meets_requirement("P1", "P1"));
        assert!(assurance_meets_requirement("P2", "P1"));
        assert!(assurance_meets_requirement("P3", "P2"));
        assert!(!assurance_meets_requirement("P1", "P2"));
        assert!(!assurance_meets_requirement("P2", "P3"));
    }
}
