use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// SHA-256 hash digest (32 bytes).
pub type Hash256 = [u8; 32];

/// Event types that can be logged to the Merkle tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// KMS key operation (generate, rotate, destroy)
    KmsOp,
    /// Credential issued to holder
    CredentialIssue,
    /// Credential revoked
    CredentialRevoke,
    /// Signing ceremony state transition
    CeremonyTransition,
    /// Gate access decision (grant/deny)
    GateDecision,
    /// Trust registry update
    TrustRegistryUpdate,
    /// Status list update
    StatusListUpdate,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KmsOp => write!(f, "kms_op"),
            Self::CredentialIssue => write!(f, "credential_issue"),
            Self::CredentialRevoke => write!(f, "credential_revoke"),
            Self::CeremonyTransition => write!(f, "ceremony_transition"),
            Self::GateDecision => write!(f, "gate_decision"),
            Self::TrustRegistryUpdate => write!(f, "trust_registry_update"),
            Self::StatusListUpdate => write!(f, "status_list_update"),
        }
    }
}

/// A single entry in the append-only Merkle log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleLogEntry {
    /// ULID with LOG_ prefix
    pub entry_id: String,
    /// Monotonic, gapless sequence number
    pub sequence: u64,
    /// When this entry was created
    pub timestamp: DateTime<Utc>,
    /// Type of event being logged
    pub event_type: EventType,
    /// SHA-256 hash of the event payload (payload itself stored elsewhere)
    pub payload_hash: Hash256,
    /// Root hash before this entry was appended
    pub previous_root: Hash256,
    /// Root hash after this entry was appended
    pub new_root: Hash256,
    /// Tenant that owns this event
    pub tenant_id: Option<String>,
}

/// Inclusion proof: proves a leaf is part of the tree at a given root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    /// The leaf hash being proved
    pub leaf_hash: Hash256,
    /// Sequence number of the leaf
    pub leaf_index: u64,
    /// Total number of leaves in the tree when proof was generated
    pub tree_size: u64,
    /// Sibling hashes from leaf to root (bottom-up)
    pub proof_hashes: Vec<Hash256>,
    /// The root hash this proof resolves to
    pub root_hash: Hash256,
}

/// Consistency proof: proves old tree is a prefix of new tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyProof {
    pub old_size: u64,
    pub new_size: u64,
    pub old_root: Hash256,
    pub new_root: Hash256,
    pub proof_hashes: Vec<Hash256>,
}
