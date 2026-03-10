use crypto_engine::error::{CryptoError, ErrorCode};
use crypto_engine::kms::{
    DestroyConfirmation, KeyAlgorithm, KeyState, KmsProvider, SoftwareKmsProvider,
};

fn provider() -> SoftwareKmsProvider {
    SoftwareKmsProvider::new()
}

// ─── KEY GENERATION ─────────────────────────────────────────────────────

#[tokio::test]
async fn generate_ed25519_key() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "test-issuer", Some("TNT_001"))
        .await
        .unwrap();

    assert!(handle.id.starts_with("KEY_"));

    let meta = kms.get_key_metadata(&handle).await.unwrap();
    assert_eq!(meta.algorithm, KeyAlgorithm::Ed25519);
    assert_eq!(meta.state, KeyState::Active);
    assert_eq!(meta.label, "test-issuer");
    assert_eq!(meta.tenant_id.as_deref(), Some("TNT_001"));
}

#[tokio::test]
async fn generate_ecdsa_p256_key() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::EcdsaP256, "test-pades", None)
        .await
        .unwrap();

    assert!(handle.id.starts_with("KEY_"));

    let meta = kms.get_key_metadata(&handle).await.unwrap();
    assert_eq!(meta.algorithm, KeyAlgorithm::EcdsaP256);
    assert_eq!(meta.state, KeyState::Active);
}

#[tokio::test]
async fn generate_multiple_keys_unique_handles() {
    let kms = provider();
    let h1 = kms
        .generate_key(KeyAlgorithm::Ed25519, "key-1", None)
        .await
        .unwrap();
    let h2 = kms
        .generate_key(KeyAlgorithm::Ed25519, "key-2", None)
        .await
        .unwrap();

    assert_ne!(h1.id, h2.id);
}

// ─── SIGN & VERIFY ──────────────────────────────────────────────────────

#[tokio::test]
async fn ed25519_sign_and_verify_roundtrip() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "signer", None)
        .await
        .unwrap();

    let data = b"credential payload for SD-JWT";
    let sig = kms.sign(&handle, data).await.unwrap();

    assert_eq!(sig.algorithm, KeyAlgorithm::Ed25519);
    assert_eq!(sig.bytes.len(), 64); // Ed25519 signature is 64 bytes

    let valid = kms.verify(&handle, data, &sig).await.unwrap();
    assert!(valid);
}

#[tokio::test]
async fn ecdsa_p256_sign_and_verify_roundtrip() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::EcdsaP256, "doc-signer", None)
        .await
        .unwrap();

    let data = b"PAdES document hash";
    let sig = kms.sign(&handle, data).await.unwrap();

    assert_eq!(sig.algorithm, KeyAlgorithm::EcdsaP256);
    assert!(!sig.bytes.is_empty());

    let valid = kms.verify(&handle, data, &sig).await.unwrap();
    assert!(valid);
}

#[tokio::test]
async fn verify_rejects_tampered_data() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "signer", None)
        .await
        .unwrap();

    let data = b"original data";
    let sig = kms.sign(&handle, data).await.unwrap();

    let tampered = b"tampered data";
    let valid = kms.verify(&handle, tampered, &sig).await.unwrap();
    assert!(!valid, "Tampered data must not verify");
}

#[tokio::test]
async fn verify_rejects_wrong_key() {
    let kms = provider();
    let key_a = kms
        .generate_key(KeyAlgorithm::Ed25519, "key-a", None)
        .await
        .unwrap();
    let key_b = kms
        .generate_key(KeyAlgorithm::Ed25519, "key-b", None)
        .await
        .unwrap();

    let data = b"some data";
    let sig = kms.sign(&key_a, data).await.unwrap();

    let valid = kms.verify(&key_b, data, &sig).await.unwrap();
    assert!(!valid, "Signature from key A must not verify with key B");
}

#[tokio::test]
async fn sign_fails_for_nonexistent_key() {
    let kms = provider();
    let fake_handle = crypto_engine::kms::KeyHandle::new("KEY_nonexistent".to_string());

    let result = kms.sign(&fake_handle, b"data").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CryptoError::Kms { code, .. } => {
            assert_eq!(code, ErrorCode::KeyNotFound);
        }
        other => panic!("Expected Kms error, got: {other:?}"),
    }
}

// ─── PUBLIC KEY EXPORT ──────────────────────────────────────────────────

#[tokio::test]
async fn export_public_key_ed25519() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "export-test", None)
        .await
        .unwrap();

    let pub_key = kms.export_public_key(&handle).await.unwrap();
    assert_eq!(pub_key.algorithm, KeyAlgorithm::Ed25519);
    assert_eq!(pub_key.bytes.len(), 32); // Ed25519 public key is 32 bytes
}

#[tokio::test]
async fn export_public_key_ecdsa() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::EcdsaP256, "export-test", None)
        .await
        .unwrap();

    let pub_key = kms.export_public_key(&handle).await.unwrap();
    assert_eq!(pub_key.algorithm, KeyAlgorithm::EcdsaP256);
    assert!(!pub_key.bytes.is_empty());
}

// ─── KEY ROTATION ───────────────────────────────────────────────────────

#[tokio::test]
async fn rotate_key_creates_new_active_key() {
    let kms = provider();
    let old_handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "rotate-test", Some("TNT_001"))
        .await
        .unwrap();

    let rotation = kms.rotate_key(&old_handle).await.unwrap();

    assert_ne!(rotation.new_handle.id, rotation.old_handle.id);
    assert_eq!(rotation.old_handle.id, old_handle.id);

    // New key is Active
    let new_meta = kms.get_key_metadata(&rotation.new_handle).await.unwrap();
    assert_eq!(new_meta.state, KeyState::Active);
    assert_eq!(new_meta.algorithm, KeyAlgorithm::Ed25519);

    // Old key is VerifyOnly
    let old_meta = kms.get_key_metadata(&old_handle).await.unwrap();
    assert_eq!(old_meta.state, KeyState::VerifyOnly);
    assert!(old_meta.rotated_at.is_some());
    assert!(old_meta.expires_at.is_some());
}

#[tokio::test]
async fn rotated_key_can_still_verify() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "rotate-verify", None)
        .await
        .unwrap();

    // Sign with original key
    let data = b"signed before rotation";
    let sig = kms.sign(&handle, data).await.unwrap();

    // Rotate
    kms.rotate_key(&handle).await.unwrap();

    // Old key can still verify (VerifyOnly state)
    let valid = kms.verify(&handle, data, &sig).await.unwrap();
    assert!(valid, "VerifyOnly key must still verify existing signatures");
}

#[tokio::test]
async fn rotated_key_cannot_sign() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "rotate-sign", None)
        .await
        .unwrap();

    kms.rotate_key(&handle).await.unwrap();

    // Old key cannot sign (VerifyOnly)
    let result = kms.sign(&handle, b"new data").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CryptoError::Kms { code, .. } => {
            assert_eq!(code, ErrorCode::InvalidKeyState);
        }
        other => panic!("Expected InvalidKeyState, got: {other:?}"),
    }
}

#[tokio::test]
async fn rotate_nonexistent_key_fails() {
    let kms = provider();
    let fake = crypto_engine::kms::KeyHandle::new("KEY_nonexistent".to_string());

    let result = kms.rotate_key(&fake).await;
    assert!(result.is_err());
}

// ─── KEY DESTRUCTION ────────────────────────────────────────────────────

#[tokio::test]
async fn destroy_key_with_valid_confirmation() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "destroy-test", None)
        .await
        .unwrap();

    let confirmation = DestroyConfirmation {
        key_id: handle.id.clone(),
        confirmation_phrase: format!("DESTROY {}", handle.id),
        actor_id: "USR_admin".to_string(),
    };

    kms.destroy_key(&handle, confirmation).await.unwrap();

    // Key metadata shows Destroyed state
    let meta = kms.get_key_metadata(&handle).await.unwrap();
    assert_eq!(meta.state, KeyState::Destroyed);
}

#[tokio::test]
async fn destroy_key_with_wrong_confirmation_fails() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "destroy-fail", None)
        .await
        .unwrap();

    let bad_confirmation = DestroyConfirmation {
        key_id: handle.id.clone(),
        confirmation_phrase: "WRONG PHRASE".to_string(),
        actor_id: "USR_admin".to_string(),
    };

    let result = kms.destroy_key(&handle, bad_confirmation).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CryptoError::Kms { code, .. } => {
            assert_eq!(code, ErrorCode::DestroyConfirmationFailed);
        }
        other => panic!("Expected DestroyConfirmationFailed, got: {other:?}"),
    }

    // Key should still be Active
    let meta = kms.get_key_metadata(&handle).await.unwrap();
    assert_eq!(meta.state, KeyState::Active);
}

#[tokio::test]
async fn destroyed_key_cannot_sign() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "destroy-sign", None)
        .await
        .unwrap();

    let confirmation = DestroyConfirmation {
        key_id: handle.id.clone(),
        confirmation_phrase: format!("DESTROY {}", handle.id),
        actor_id: "USR_admin".to_string(),
    };

    kms.destroy_key(&handle, confirmation).await.unwrap();

    let result = kms.sign(&handle, b"data").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn destroyed_key_cannot_verify() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "destroy-verify", None)
        .await
        .unwrap();

    let data = b"test";
    let sig = kms.sign(&handle, data).await.unwrap();

    let confirmation = DestroyConfirmation {
        key_id: handle.id.clone(),
        confirmation_phrase: format!("DESTROY {}", handle.id),
        actor_id: "USR_admin".to_string(),
    };

    kms.destroy_key(&handle, confirmation).await.unwrap();

    let result = kms.verify(&handle, data, &sig).await;
    assert!(result.is_err());
}

// ─── LIST KEYS ──────────────────────────────────────────────────────────

#[tokio::test]
async fn list_keys_returns_all() {
    let kms = provider();
    kms.generate_key(KeyAlgorithm::Ed25519, "key-1", None)
        .await
        .unwrap();
    kms.generate_key(KeyAlgorithm::EcdsaP256, "key-2", None)
        .await
        .unwrap();

    let keys = kms.list_keys(None).await.unwrap();
    assert_eq!(keys.len(), 2);
}

#[tokio::test]
async fn list_keys_filters_by_tenant() {
    let kms = provider();
    kms.generate_key(KeyAlgorithm::Ed25519, "tenant-a", Some("TNT_A"))
        .await
        .unwrap();
    kms.generate_key(KeyAlgorithm::Ed25519, "tenant-b", Some("TNT_B"))
        .await
        .unwrap();
    kms.generate_key(KeyAlgorithm::Ed25519, "no-tenant", None)
        .await
        .unwrap();

    let keys_a = kms.list_keys(Some("TNT_A")).await.unwrap();
    assert_eq!(keys_a.len(), 1);
    assert_eq!(keys_a[0].tenant_id.as_deref(), Some("TNT_A"));

    let keys_all = kms.list_keys(None).await.unwrap();
    assert_eq!(keys_all.len(), 3);
}

// ─── AUDIT EVENTS ───────────────────────────────────────────────────────

#[tokio::test]
async fn audit_events_emitted_for_all_operations() {
    let kms = provider();

    // Generate
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "audit-test", Some("TNT_001"))
        .await
        .unwrap();

    // Sign
    let sig = kms.sign(&handle, b"data").await.unwrap();

    // Verify
    kms.verify(&handle, b"data", &sig).await.unwrap();

    // Export public key
    kms.export_public_key(&handle).await.unwrap();

    // List
    kms.list_keys(None).await.unwrap();

    // Rotate
    kms.rotate_key(&handle).await.unwrap();

    let events = kms.drain_audit_events().await;

    // generate (original) + sign + verify + export + list + generate (rotation) + rotate
    assert!(events.len() >= 7, "Expected at least 7 audit events, got {}", events.len());

    // All events should be successful
    assert!(events.iter().all(|e| e.success));

    // All events should have event IDs with EVT_ prefix
    assert!(events.iter().all(|e| e.event_id.starts_with("EVT_")));
}

#[tokio::test]
async fn audit_event_on_failure() {
    let kms = provider();
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "audit-fail", None)
        .await
        .unwrap();

    let bad_confirmation = DestroyConfirmation {
        key_id: handle.id.clone(),
        confirmation_phrase: "WRONG".to_string(),
        actor_id: "USR_bad_actor".to_string(),
    };

    let _ = kms.destroy_key(&handle, bad_confirmation).await;

    let events = kms.drain_audit_events().await;
    let failure_events: Vec<_> = events.iter().filter(|e| !e.success).collect();

    assert_eq!(failure_events.len(), 1);
    assert_eq!(
        failure_events[0].error_code.as_deref(),
        Some("SAHI_2010")
    );
}

// ─── CONCURRENT ACCESS ─────────────────────────────────────────────────

#[tokio::test]
async fn concurrent_key_generation() {
    let kms = std::sync::Arc::new(provider());
    let mut handles = Vec::new();

    for i in 0..10 {
        let kms = kms.clone();
        handles.push(tokio::spawn(async move {
            kms.generate_key(KeyAlgorithm::Ed25519, &format!("concurrent-{i}"), None)
                .await
                .unwrap()
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All keys should have unique IDs
    let unique_ids: std::collections::HashSet<_> = results.iter().map(|h| &h.id).collect();
    assert_eq!(unique_ids.len(), 10);

    // All keys should be listable
    let keys = kms.list_keys(None).await.unwrap();
    assert_eq!(keys.len(), 10);
}

#[tokio::test]
async fn concurrent_sign_verify() {
    let kms = std::sync::Arc::new(provider());
    let handle = kms
        .generate_key(KeyAlgorithm::Ed25519, "concurrent-sign", None)
        .await
        .unwrap();

    let mut tasks = Vec::new();
    for i in 0..20 {
        let kms = kms.clone();
        let handle = handle.clone();
        tasks.push(tokio::spawn(async move {
            let data = format!("message-{i}");
            let sig = kms.sign(&handle, data.as_bytes()).await.unwrap();
            let valid = kms.verify(&handle, data.as_bytes(), &sig).await.unwrap();
            assert!(valid, "Concurrent verify failed for message-{i}");
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}
