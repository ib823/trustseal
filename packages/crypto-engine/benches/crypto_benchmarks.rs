//! Performance benchmarks for crypto-engine.
//!
//! Critical targets (from CLAUDE.md):
//! - SD-JWT verification: < 50ms (crypto only, no network)
//!
//! Run with: `cargo bench -p crypto-engine`

use std::sync::Arc;

use chrono::{Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use crypto_engine::kms::{KeyAlgorithm, KmsProvider, SoftwareKmsProvider};
use crypto_engine::merkle::{EventType, MerkleTree};
use crypto_engine::sd_jwt::{
    ClaimPath, CredentialSubject, IssuanceOptions, SdJwtIssuer, SdJwtVerifier, VaultPassCredential,
};
use sd_jwt_payload::SdJwt;
use tokio::runtime::Runtime;

/// Benchmark KMS Ed25519 signing operations.
fn bench_kms_ed25519_signing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let kms = SoftwareKmsProvider::new();
    let kms: Arc<dyn KmsProvider> = Arc::new(kms);

    // Generate a key for benchmarking
    let key_handle = rt.block_on(async {
        kms.generate_key(KeyAlgorithm::Ed25519, "bench-key", None)
            .await
            .unwrap()
    });

    let message = b"Benchmark message for signing operation - VaultPass credential payload";

    let mut group = c.benchmark_group("kms_ed25519");
    group.throughput(Throughput::Bytes(message.len() as u64));

    group.bench_function("sign", |b| {
        b.iter(|| {
            rt.block_on(async {
                let sig = kms.sign(&key_handle, black_box(message)).await.unwrap();
                black_box(sig)
            })
        });
    });

    // Get signature for verify benchmark
    let signature = rt
        .block_on(async { kms.sign(&key_handle, message).await })
        .unwrap();

    group.bench_function("verify", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = kms
                    .verify(&key_handle, black_box(message), black_box(&signature))
                    .await
                    .unwrap();
                black_box(result)
            })
        });
    });

    group.finish();
}

/// Benchmark Merkle tree operations.
fn bench_merkle_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_tree");

    // Build tree with 100 entries
    group.bench_function("append_100_entries", |b| {
        b.iter(|| {
            let mut tree = MerkleTree::new();
            for i in 0..100 {
                let payload = format!("entry_{i}");
                tree.append(EventType::CredentialIssue, payload.as_bytes(), None);
            }
            black_box(tree)
        });
    });

    // Build tree with 1000 entries
    group.bench_function("append_1000_entries", |b| {
        b.iter(|| {
            let mut tree = MerkleTree::new();
            for i in 0..1000 {
                let payload = format!("entry_{i}");
                tree.append(EventType::CredentialIssue, payload.as_bytes(), None);
            }
            black_box(tree)
        });
    });

    // Proof generation from 1000-entry tree
    let mut tree = MerkleTree::new();
    for i in 0..1000 {
        let payload = format!("entry_{i}");
        tree.append(EventType::CredentialIssue, payload.as_bytes(), None);
    }

    group.bench_function("generate_proof_1000_entries", |b| {
        b.iter(|| {
            let proof = tree.inclusion_proof(black_box(500)).unwrap();
            black_box(proof)
        });
    });

    // Proof verification
    let proof = tree.inclusion_proof(500).unwrap();
    group.bench_function("verify_proof_1000_entries", |b| {
        b.iter(|| {
            let valid = MerkleTree::verify_inclusion(black_box(&proof));
            black_box(valid)
        });
    });

    group.finish();
}

/// Benchmark SD-JWT verification (critical path: < 50ms target).
fn bench_sd_jwt_verification(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Set up issuer with KMS
    let kms = SoftwareKmsProvider::new();
    let kms: Arc<dyn KmsProvider> = Arc::new(kms);
    let issuer = SdJwtIssuer::new(kms.clone());

    // Generate issuer key
    let issuer_key = rt.block_on(async {
        kms.generate_key(KeyAlgorithm::Ed25519, "issuer-bench-key", None)
            .await
            .unwrap()
    });

    // Create a realistic credential
    let credential = VaultPassCredential {
        context: vec![
            "https://www.w3.org/ns/credentials/v2".to_string(),
            "https://sahi.my/ns/vaultpass/v1".to_string(),
        ],
        credential_type: vec![
            "VerifiableCredential".to_string(),
            "ResidentBadge".to_string(),
        ],
        issuer: "did:web:sahi.my".to_string(),
        valid_from: Utc::now(),
        valid_until: Some(Utc::now() + Duration::days(365)),
        credential_subject: CredentialSubject {
            id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            property_id: "PRY_01HXK123456789ABCDEF".to_string(),
            unit: Some("A-12-03".to_string()),
            name: Some("Ahmad bin Ibrahim".to_string()),
            role: "resident".to_string(),
            access_zones: vec![
                "lobby".to_string(),
                "parking".to_string(),
                "gym".to_string(),
            ],
            time_restrictions: None,
        },
        credential_status: None,
    };

    let issuance_options = IssuanceOptions {
        key_handle: issuer_key.id.clone(),
        concealable_claims: vec![
            ClaimPath::from("/credentialSubject/name"),
            ClaimPath::from("/credentialSubject/unit"),
        ],
        decoy_count: 3,
        holder_public_key: None,
    };

    // Issue the credential
    let sd_jwt = rt
        .block_on(async { issuer.issue(&credential, issuance_options).await })
        .unwrap();

    // Get the public key for verification
    let public_key = rt
        .block_on(async { kms.export_public_key(&issuer_key).await })
        .unwrap();

    let verifier = SdJwtVerifier::new().with_max_kb_jwt_age(300);

    let mut group = c.benchmark_group("sd_jwt_verification");

    // This is the critical benchmark - must be < 50ms
    group.bench_function("verify_credential_no_kb", |b| {
        b.iter(|| {
            let result =
                verifier.verify(black_box(&sd_jwt), black_box(public_key.as_bytes()), None);
            black_box(result)
        });
    });

    // Parse SD-JWT from string (common path)
    let sd_jwt_str = sd_jwt.to_string();
    group.bench_function("parse_sd_jwt", |b| {
        b.iter(|| {
            let parsed: SdJwt = black_box(&sd_jwt_str).parse().unwrap();
            black_box(parsed)
        });
    });

    // Full parse + verify (end-to-end)
    group.bench_function("parse_and_verify", |b| {
        b.iter(|| {
            let parsed: SdJwt = black_box(&sd_jwt_str).parse().unwrap();
            let result = verifier.verify(&parsed, public_key.as_bytes(), None);
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark KMS key generation.
fn bench_kms_key_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("kms_key_generation");

    group.bench_function("ed25519", |b| {
        b.iter(|| {
            rt.block_on(async {
                let kms = SoftwareKmsProvider::new();
                let key = kms
                    .generate_key(KeyAlgorithm::Ed25519, "bench-ed25519", None)
                    .await
                    .unwrap();
                black_box(key)
            })
        });
    });

    group.bench_function("ecdsa_p256", |b| {
        b.iter(|| {
            rt.block_on(async {
                let kms = SoftwareKmsProvider::new();
                let key = kms
                    .generate_key(KeyAlgorithm::EcdsaP256, "bench-p256", None)
                    .await
                    .unwrap();
                black_box(key)
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_kms_key_generation,
    bench_kms_ed25519_signing,
    bench_merkle_tree,
    bench_sd_jwt_verification,
);

criterion_main!(benches);
