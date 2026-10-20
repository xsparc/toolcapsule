use ed25519_dalek::SigningKey;
use std::sync::Arc;
use toolcapsule::{audit::Receipt, cache::PreparationCache, package::digest, *};

fn bytes() -> Vec<u8> {
    wat::parse_str(
        r#"(component
        (core module $m (global $n (mut i32) (i32.const 0))
          (func (export "run") (result i32)
            global.get $n i32.const 1 global.set $n))
        (core instance $i (instantiate $m))
        (func $run (result (result)) (canon lift (core func $i "run")))
        (instance $cli (export "run" (func $run)))
        (export "wasi:cli/run@0.2.0" (instance $cli)))"#,
    )
    .unwrap()
}
fn fixture(bytes: &[u8], version: &str, signer: &str) -> (Vec<u8>, TrustStore, Policy) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let manifest = Manifest {
        schema_version: 1,
        name: "cache-fixture".into(),
        version: version.into(),
        description: "Synthetic cache fixture".into(),
        abi: "wasi:cli/command@0.2".into(),
        component_sha256: digest(bytes),
        capabilities: Capabilities::default(),
        limits: Limits::default(),
    };
    (
        serde_json::to_vec(&SignedManifest::sign(&manifest, signer, &key).unwrap()).unwrap(),
        [(signer.into(), hex::encode(key.verifying_key().to_bytes()))].into(),
        Policy {
            allowed_tools: vec!["cache-fixture".into()],
            ..Policy::default()
        },
    )
}

#[tokio::test]
async fn cache_hit_reuses_code_with_fresh_guest_state_and_replay() {
    let bytes = bytes();
    let (envelope, trust, policy) = fixture(&bytes, "1", "one");
    let mut cache = PreparationCache::new(2, 4096).unwrap();
    let first = cache
        .prepare(&envelope, bytes.clone(), &trust, &policy)
        .unwrap();
    let second = cache
        .prepare(&envelope, bytes.clone(), &trust, &policy)
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    let result = first.run(b"", 1).await.unwrap();
    assert_eq!(result.outcome, runtime::Outcome::Success);
    Receipt::new(&first, b"", 1, &result)
        .replay(&second, b"")
        .await
        .unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.source_bytes(), bytes.len());
    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.source_bytes(), 0);
    assert_eq!(
        second.run(b"", 1).await.unwrap().outcome,
        runtime::Outcome::Success
    );
}

#[test]
fn hits_never_bypass_current_signature_digest_or_policy_checks() {
    let bytes = bytes();
    let (envelope, trust, policy) = fixture(&bytes, "1", "one");
    let mut cache = PreparationCache::new(2, 4096).unwrap();
    cache
        .prepare(&envelope, bytes.clone(), &trust, &policy)
        .unwrap();
    assert!(matches!(
        cache.prepare(&envelope, bytes.clone(), &TrustStore::new(), &policy),
        Err(Error::Signature)
    ));
    let rotated = [(
        "one".into(),
        hex::encode(SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes()),
    )]
    .into();
    assert!(matches!(
        cache.prepare(&envelope, bytes.clone(), &rotated, &policy),
        Err(Error::Signature)
    ));
    assert!(matches!(
        cache.prepare(&envelope, bytes.clone(), &trust, &Policy::default()),
        Err(Error::Policy)
    ));
    let mut restricted = policy.clone();
    restricted.max_limits.fuel = 1;
    assert!(matches!(
        cache.prepare(&envelope, bytes.clone(), &trust, &restricted),
        Err(Error::Policy)
    ));
    let mut corrupt = bytes.clone();
    corrupt.push(0);
    assert!(matches!(
        cache.prepare(&envelope, corrupt, &trust, &policy),
        Err(Error::Digest)
    ));
    let mut forged: SignedManifest = serde_json::from_slice(&envelope).unwrap();
    forged.signature = "00".repeat(64);
    assert!(matches!(
        cache.prepare(
            &serde_json::to_vec(&forged).unwrap(),
            bytes,
            &trust,
            &policy
        ),
        Err(Error::Signature)
    ));
    assert_eq!(cache.len(), 1);
}

#[test]
fn lru_eviction_manifest_identity_and_failed_compilation() {
    let bytes = bytes();
    let (a, trust, policy) = fixture(&bytes, "1", "one");
    let (b, _, _) = fixture(&bytes, "2", "one");
    let (c, _, _) = fixture(&bytes, "3", "one");
    let mut cache = PreparationCache::new(2, bytes.len() * 2).unwrap();
    let first = cache.prepare(&a, bytes.clone(), &trust, &policy).unwrap();
    let second = cache.prepare(&b, bytes.clone(), &trust, &policy).unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert!(Arc::ptr_eq(
        &first,
        &cache.prepare(&a, bytes.clone(), &trust, &policy).unwrap()
    ));
    cache.prepare(&c, bytes.clone(), &trust, &policy).unwrap();
    assert_eq!(Arc::strong_count(&second), 1); // B, not recently touched A, was evicted.
    assert_eq!(cache.source_bytes(), bytes.len() * 2);
    let invalid = b"not a component".to_vec();
    let (envelope, _, _) = fixture(&invalid, "bad", "one");
    assert!(matches!(
        cache.prepare(&envelope, invalid, &trust, &policy),
        Err(Error::Component)
    ));
    assert!(Arc::ptr_eq(
        &first,
        &cache.prepare(&a, bytes.clone(), &trust, &policy).unwrap()
    ));
    let (other_signer, other_trust, _) = fixture(&bytes, "1", "two");
    let other = cache
        .prepare(&other_signer, bytes, &other_trust, &policy)
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &other));
}

#[test]
fn source_budget_and_configuration_are_bounded() {
    for (count, size) in [(0, 1), (65, 1), (1, 0), (1, 256 * 1024 * 1024 + 1)] {
        assert!(PreparationCache::new(count, size).is_err());
    }
    let bytes = bytes();
    let (a, trust, policy) = fixture(&bytes, "1", "one");
    let (b, _, _) = fixture(&bytes, "2", "one");
    let mut small = PreparationCache::new(2, bytes.len() - 1).unwrap();
    assert!(matches!(
        small.prepare(&a, bytes.clone(), &trust, &policy),
        Err(Error::Policy)
    ));
    assert!(small.is_empty());
    let mut cache = PreparationCache::new(64, bytes.len()).unwrap();
    let first = cache.prepare(&a, bytes.clone(), &trust, &policy).unwrap();
    cache.prepare(&b, bytes.clone(), &trust, &policy).unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.source_bytes(), bytes.len());
    assert_eq!(Arc::strong_count(&first), 1);
}
