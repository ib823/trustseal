use crypto_engine::merkle::{EventType, MerkleTree};

// ─── APPEND ─────────────────────────────────────────────────────────────

#[test]
fn empty_tree_has_zero_root() {
    let tree = MerkleTree::new();
    assert_eq!(tree.root(), [0u8; 32]);
    assert_eq!(tree.size(), 0);
}

#[test]
fn append_single_entry() {
    let mut tree = MerkleTree::new();
    let entry = tree.append(EventType::KmsOp, b"key generated", Some("TNT_001"));

    assert!(entry.entry_id.starts_with("LOG_"));
    assert_eq!(entry.sequence, 0);
    assert_eq!(entry.event_type, EventType::KmsOp);
    assert_eq!(entry.previous_root, [0u8; 32]);
    assert_ne!(entry.new_root, [0u8; 32]);
    assert_eq!(entry.tenant_id.as_deref(), Some("TNT_001"));
    assert_eq!(tree.size(), 1);
}

#[test]
fn append_multiple_entries_monotonic_sequence() {
    let mut tree = MerkleTree::new();

    for i in 0..10 {
        let entry = tree.append(
            EventType::GateDecision,
            format!("event-{i}").as_bytes(),
            None,
        );
        assert_eq!(entry.sequence, i);
    }

    assert_eq!(tree.size(), 10);
}

#[test]
fn each_append_changes_root() {
    let mut tree = MerkleTree::new();
    let mut roots = Vec::new();

    for i in 0..5 {
        tree.append(
            EventType::CredentialIssue,
            format!("cred-{i}").as_bytes(),
            None,
        );
        roots.push(tree.root());
    }

    // All roots should be unique
    for (i, r) in roots.iter().enumerate() {
        for (j, s) in roots.iter().enumerate() {
            if i != j {
                assert_ne!(
                    r, s,
                    "Root at index {i} should differ from root at index {j}"
                );
            }
        }
    }
}

#[test]
fn previous_root_chains_correctly() {
    let mut tree = MerkleTree::new();

    let e1 = tree.append(EventType::KmsOp, b"first", None);
    let e2 = tree.append(EventType::KmsOp, b"second", None);
    let e3 = tree.append(EventType::KmsOp, b"third", None);

    assert_eq!(e1.previous_root, [0u8; 32]);
    assert_eq!(e2.previous_root, e1.new_root);
    assert_eq!(e3.previous_root, e2.new_root);
}

#[test]
fn deterministic_root_for_same_data() {
    let mut tree1 = MerkleTree::new();
    let mut tree2 = MerkleTree::new();

    // Use fixed payloads — the root depends only on payload content, not timestamps
    // Since entries include timestamps and ULIDs, root won't match between trees.
    // But the TREE root (computed from leaf hashes of payload_hash) should be deterministic
    // for the same payload data in the same order.
    tree1.append(EventType::KmsOp, b"payload-a", None);
    tree2.append(EventType::KmsOp, b"payload-a", None);

    // Root is computed from hash(hash(payload)), which is deterministic
    assert_eq!(tree1.root(), tree2.root());
}

// ─── INCLUSION PROOFS ───────────────────────────────────────────────────

#[test]
fn inclusion_proof_single_leaf() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"only-leaf", None);

    let proof = tree.inclusion_proof(0).unwrap();
    assert!(MerkleTree::verify_inclusion(&proof));
}

#[test]
fn inclusion_proof_two_leaves() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"leaf-0", None);
    tree.append(EventType::KmsOp, b"leaf-1", None);

    let proof0 = tree.inclusion_proof(0).unwrap();
    assert!(MerkleTree::verify_inclusion(&proof0));

    let proof1 = tree.inclusion_proof(1).unwrap();
    assert!(MerkleTree::verify_inclusion(&proof1));
}

#[test]
fn inclusion_proof_power_of_two_leaves() {
    let mut tree = MerkleTree::new();
    for i in 0..8 {
        tree.append(
            EventType::GateDecision,
            format!("leaf-{i}").as_bytes(),
            None,
        );
    }

    for i in 0..8 {
        let proof = tree.inclusion_proof(i).unwrap();
        assert!(
            MerkleTree::verify_inclusion(&proof),
            "Inclusion proof failed for leaf {i}"
        );
    }
}

#[test]
fn inclusion_proof_non_power_of_two_leaves() {
    let mut tree = MerkleTree::new();
    for i in 0..5 {
        tree.append(
            EventType::CredentialIssue,
            format!("leaf-{i}").as_bytes(),
            None,
        );
    }

    for i in 0..5 {
        let proof = tree.inclusion_proof(i).unwrap();
        assert!(
            MerkleTree::verify_inclusion(&proof),
            "Inclusion proof failed for leaf {i}"
        );
    }
}

#[test]
fn inclusion_proof_large_tree() {
    let mut tree = MerkleTree::new();
    for i in 0..100 {
        tree.append(
            EventType::GateDecision,
            format!("event-{i}").as_bytes(),
            None,
        );
    }

    // Verify first, middle, and last
    for idx in [0, 49, 99] {
        let proof = tree.inclusion_proof(idx).unwrap();
        assert!(
            MerkleTree::verify_inclusion(&proof),
            "Inclusion proof failed for leaf {idx}"
        );
    }
}

#[test]
fn inclusion_proof_out_of_range() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"only", None);

    let result = tree.inclusion_proof(1);
    assert!(result.is_err());
}

#[test]
fn tampered_proof_fails_verification() {
    let mut tree = MerkleTree::new();
    for i in 0..4 {
        tree.append(EventType::KmsOp, format!("leaf-{i}").as_bytes(), None);
    }

    let mut proof = tree.inclusion_proof(2).unwrap();

    // Tamper with the leaf hash
    proof.leaf_hash[0] ^= 0xFF;
    assert!(
        !MerkleTree::verify_inclusion(&proof),
        "Tampered leaf should fail verification"
    );
}

#[test]
fn tampered_proof_hash_fails_verification() {
    let mut tree = MerkleTree::new();
    for i in 0..4 {
        tree.append(EventType::KmsOp, format!("leaf-{i}").as_bytes(), None);
    }

    let mut proof = tree.inclusion_proof(1).unwrap();

    // Tamper with a proof hash
    if let Some(h) = proof.proof_hashes.first_mut() {
        h[0] ^= 0xFF;
    }
    assert!(
        !MerkleTree::verify_inclusion(&proof),
        "Tampered proof hash should fail"
    );
}

// ─── CONSISTENCY PROOFS ─────────────────────────────────────────────────

#[test]
fn consistency_proof_empty_to_nonempty() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"first", None);

    let proof = tree.consistency_proof(0).unwrap();
    assert_eq!(proof.old_size, 0);
    assert_eq!(proof.new_size, 1);
    assert!(MerkleTree::verify_consistency(&proof));
}

#[test]
fn consistency_proof_same_size() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"only", None);

    let proof = tree.consistency_proof(1).unwrap();
    assert_eq!(proof.old_size, 1);
    assert_eq!(proof.new_size, 1);
    assert_eq!(proof.old_root, proof.new_root);
    assert!(MerkleTree::verify_consistency(&proof));
}

#[test]
fn consistency_proof_growing_tree() {
    let mut tree = MerkleTree::new();
    for i in 0..4 {
        tree.append(EventType::KmsOp, format!("leaf-{i}").as_bytes(), None);
    }

    let proof = tree.consistency_proof(2).unwrap();
    assert_eq!(proof.old_size, 2);
    assert_eq!(proof.new_size, 4);
    assert!(MerkleTree::verify_consistency(&proof));
}

#[test]
fn consistency_proof_old_size_too_large() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"only", None);

    let result = tree.consistency_proof(5);
    assert!(result.is_err());
}

// ─── ENTRY RETRIEVAL ────────────────────────────────────────────────────

#[test]
fn get_entry_by_sequence() {
    let mut tree = MerkleTree::new();
    tree.append(EventType::KmsOp, b"zero", None);
    tree.append(EventType::CredentialIssue, b"one", None);
    tree.append(EventType::GateDecision, b"two", None);

    let entry = tree.get_entry(1).unwrap();
    assert_eq!(entry.sequence, 1);
    assert_eq!(entry.event_type, EventType::CredentialIssue);
}

#[test]
fn get_entry_out_of_range() {
    let tree = MerkleTree::new();
    assert!(tree.get_entry(0).is_none());
}

#[test]
fn entries_returns_all() {
    let mut tree = MerkleTree::new();
    for i in 0..5 {
        tree.append(EventType::KmsOp, format!("e-{i}").as_bytes(), None);
    }

    let entries = tree.entries();
    assert_eq!(entries.len(), 5);
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.sequence, i as u64);
    }
}

// ─── TAMPER DETECTION ───────────────────────────────────────────────────

#[test]
fn different_payloads_produce_different_roots() {
    let mut tree1 = MerkleTree::new();
    let mut tree2 = MerkleTree::new();

    tree1.append(EventType::KmsOp, b"payload-a", None);
    tree2.append(EventType::KmsOp, b"payload-b", None);

    assert_ne!(tree1.root(), tree2.root());
}

#[test]
fn order_matters() {
    let mut tree1 = MerkleTree::new();
    let mut tree2 = MerkleTree::new();

    tree1.append(EventType::KmsOp, b"first", None);
    tree1.append(EventType::KmsOp, b"second", None);

    tree2.append(EventType::KmsOp, b"second", None);
    tree2.append(EventType::KmsOp, b"first", None);

    assert_ne!(
        tree1.root(),
        tree2.root(),
        "Different order must produce different root"
    );
}

// ─── EVENT TYPES ────────────────────────────────────────────────────────

#[test]
fn all_event_types_can_be_logged() {
    let mut tree = MerkleTree::new();

    let types = vec![
        EventType::KmsOp,
        EventType::CredentialIssue,
        EventType::CredentialRevoke,
        EventType::CeremonyTransition,
        EventType::GateDecision,
        EventType::TrustRegistryUpdate,
        EventType::StatusListUpdate,
    ];

    for event_type in &types {
        tree.append(event_type.clone(), b"payload", Some("TNT_001"));
    }

    assert_eq!(tree.size(), 7);

    for (i, event_type) in types.iter().enumerate() {
        let entry = tree.get_entry(i as u64).unwrap();
        assert_eq!(&entry.event_type, event_type);
    }
}
