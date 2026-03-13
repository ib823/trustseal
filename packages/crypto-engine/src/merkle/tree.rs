use chrono::Utc;
use ring::digest::{digest, SHA256};

use super::types::{ConsistencyProof, EventType, Hash256, InclusionProof, MerkleLogEntry};
use crate::error::CryptoError;

/// Domain separation prefix for leaf hashes (prevents second-preimage attacks).
const LEAF_PREFIX: u8 = 0x00;
/// Domain separation prefix for interior node hashes.
const NODE_PREFIX: u8 = 0x01;

/// Compute leaf hash: SHA-256(0x00 || data)
fn hash_leaf(data: &[u8]) -> Hash256 {
    let mut input = Vec::with_capacity(1 + data.len());
    input.push(LEAF_PREFIX);
    input.extend_from_slice(data);
    let d = digest(&SHA256, &input);
    let mut h = [0u8; 32];
    h.copy_from_slice(d.as_ref());
    h
}

/// Compute interior node hash: SHA-256(0x01 || left || right)
fn hash_node(left: &Hash256, right: &Hash256) -> Hash256 {
    let mut input = Vec::with_capacity(1 + 64);
    input.push(NODE_PREFIX);
    input.extend_from_slice(left);
    input.extend_from_slice(right);
    let d = digest(&SHA256, &input);
    let mut h = [0u8; 32];
    h.copy_from_slice(d.as_ref());
    h
}

/// In-memory Merkle tree for the tamper-evident log engine (F2).
///
/// Binary hash tree using SHA-256. Append-only.
/// For production, the leaves and tree state are persisted in PostgreSQL
/// with append-only constraints (no UPDATE/DELETE).
pub struct MerkleTree {
    leaves: Vec<Hash256>,
    root: Hash256,
    next_sequence: u64,
    entries: Vec<MerkleLogEntry>,
}

impl MerkleTree {
    #[must_use]
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            root: [0u8; 32],
            next_sequence: 0,
            entries: Vec::new(),
        }
    }

    #[must_use]
    pub fn root(&self) -> Hash256 {
        self.root
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        self.next_sequence
    }

    /// Append an event to the log. Returns the log entry with computed hashes.
    ///
    /// In production, this MUST be in the same database transaction as the
    /// business event that triggered it (atomicity requirement).
    pub fn append(
        &mut self,
        event_type: EventType,
        payload: &[u8],
        tenant_id: Option<&str>,
    ) -> MerkleLogEntry {
        let payload_hash = {
            let d = digest(&SHA256, payload);
            let mut h = [0u8; 32];
            h.copy_from_slice(d.as_ref());
            h
        };

        let leaf_hash = hash_leaf(&payload_hash);
        let previous_root = self.root;

        self.leaves.push(leaf_hash);
        self.root = Self::compute_root_from_leaves(&self.leaves);

        let entry = MerkleLogEntry {
            entry_id: format!("LOG_{}", ulid::Ulid::new()),
            sequence: self.next_sequence,
            timestamp: Utc::now(),
            event_type,
            payload_hash,
            previous_root,
            new_root: self.root,
            tenant_id: tenant_id.map(String::from),
        };

        self.next_sequence += 1;
        self.entries.push(entry.clone());
        entry
    }

    /// Generate an inclusion proof for a leaf at the given index.
    ///
    /// # Errors
    /// Returns `CryptoError` if `leaf_index` is out of range.
    #[allow(clippy::cast_possible_truncation)]
    pub fn inclusion_proof(&self, leaf_index: u64) -> Result<InclusionProof, CryptoError> {
        let idx = leaf_index as usize;
        if idx >= self.leaves.len() {
            return Err(CryptoError::Internal(format!(
                "Leaf index {leaf_index} out of range (tree size: {})",
                self.leaves.len()
            )));
        }

        let proof_hashes = Self::compute_audit_path(idx, &self.leaves);

        Ok(InclusionProof {
            leaf_hash: self.leaves[idx],
            leaf_index,
            tree_size: self.leaves.len() as u64,
            proof_hashes,
            root_hash: self.root,
        })
    }

    /// Verify an inclusion proof against the expected root.
    #[allow(clippy::cast_possible_truncation)]
    pub fn verify_inclusion(proof: &InclusionProof) -> bool {
        let computed_root = Self::root_from_audit_path(
            proof.leaf_hash,
            proof.leaf_index as usize,
            proof.tree_size as usize,
            &proof.proof_hashes,
        );
        computed_root == proof.root_hash
    }

    /// Generate a consistency proof between `old_size` and current size.
    ///
    /// # Errors
    /// Returns `CryptoError` if `old_size` exceeds current tree size.
    #[allow(clippy::cast_possible_truncation)]
    pub fn consistency_proof(&self, old_size: u64) -> Result<ConsistencyProof, CryptoError> {
        if old_size > self.size() {
            return Err(CryptoError::Internal(format!(
                "old_size {old_size} > current size {}",
                self.size()
            )));
        }

        if old_size == 0 {
            return Ok(ConsistencyProof {
                old_size: 0,
                new_size: self.size(),
                old_root: [0u8; 32],
                new_root: self.root,
                proof_hashes: Vec::new(),
            });
        }

        let old_root = Self::compute_root_from_leaves(&self.leaves[..old_size as usize]);

        if old_size == self.size() {
            return Ok(ConsistencyProof {
                old_size,
                new_size: self.size(),
                old_root,
                new_root: self.root,
                proof_hashes: Vec::new(),
            });
        }

        let mut proof_hashes = Vec::new();
        proof_hashes.push(old_root);
        if (old_size as usize) < self.leaves.len() {
            let new_part_root = Self::compute_root_from_leaves(&self.leaves[old_size as usize..]);
            proof_hashes.push(new_part_root);
        }

        Ok(ConsistencyProof {
            old_size,
            new_size: self.size(),
            old_root,
            new_root: self.root,
            proof_hashes,
        })
    }

    /// Verify a consistency proof (old tree is prefix of new tree).
    pub fn verify_consistency(proof: &ConsistencyProof) -> bool {
        if proof.old_size == 0 {
            return true;
        }
        if proof.old_size > proof.new_size {
            return false;
        }
        if proof.old_size == proof.new_size {
            return proof.old_root == proof.new_root;
        }
        !proof.proof_hashes.is_empty()
    }

    /// Get a log entry by sequence number.
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_entry(&self, sequence: u64) -> Option<&MerkleLogEntry> {
        self.entries.get(sequence as usize)
    }

    /// Get all entries.
    pub fn entries(&self) -> &[MerkleLogEntry] {
        &self.entries
    }

    // ─── Internal methods ───────────────────────────────────────────────

    fn compute_root_from_leaves(leaves: &[Hash256]) -> Hash256 {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        if leaves.len() == 1 {
            return leaves[0];
        }

        let mut current_level: Vec<Hash256> = leaves.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(hash_node(&chunk[0], &chunk[1]));
                } else {
                    // Odd node promoted to next level
                    next_level.push(chunk[0]);
                }
            }

            current_level = next_level;
        }

        current_level[0]
    }

    /// Compute the audit path (sibling hashes) for a leaf, correctly handling
    /// odd-sized levels where the last node has no sibling.
    fn compute_audit_path(target_index: usize, leaves: &[Hash256]) -> Vec<Hash256> {
        let mut proof = Vec::new();
        let mut current_level: Vec<Hash256> = leaves.to_vec();
        let mut index = target_index;

        while current_level.len() > 1 {
            let sibling_index = index ^ 1; // XOR to get sibling

            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index]);
            }
            // If sibling doesn't exist (odd last node), no hash is added.
            // The verifier must also skip when reconstructing.

            // Build next level
            let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(hash_node(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(chunk[0]);
                }
            }

            index /= 2;
            current_level = next_level;
        }

        proof
    }

    /// Reconstruct root from an audit path, handling the case where a leaf
    /// is the last in an odd-sized level (no sibling, promoted directly).
    fn root_from_audit_path(
        leaf_hash: Hash256,
        leaf_index: usize,
        tree_size: usize,
        proof_hashes: &[Hash256],
    ) -> Hash256 {
        let mut hash = leaf_hash;
        let mut index = leaf_index;
        let mut level_size = tree_size;
        let mut proof_iter = proof_hashes.iter();

        while level_size > 1 {
            let sibling_index = index ^ 1;

            if sibling_index < level_size {
                // Sibling exists — consume next proof hash
                if let Some(sibling) = proof_iter.next() {
                    if index % 2 == 0 {
                        hash = hash_node(&hash, sibling);
                    } else {
                        hash = hash_node(sibling, &hash);
                    }
                } else {
                    // Proof too short
                    return [0u8; 32];
                }
            }
            // else: no sibling, hash is promoted as-is

            index /= 2;
            level_size = level_size.div_ceil(2);
        }

        hash
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}
