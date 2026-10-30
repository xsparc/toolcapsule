use ed25519_dalek::SigningKey;
use toolcapsule::{package::digest, *};

fn fixture() -> (Manifest, SigningKey, TrustStore, Policy) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let manifest = Manifest {
        schema_version: 1,
        name: "echo".into(),
        version: "1.0.0".into(),
        description: "Synthetic fixture".into(),
        abi: "wasi:cli/command@0.2".into(),
        component_sha256: digest(b"component"),
        capabilities: Capabilities::default(),
        limits: Limits::default(),
    };
    let trust = [("test".into(), hex::encode(key.verifying_key().to_bytes()))].into();
    let policy = Policy {
        allowed_tools: vec!["echo".into()],
        ..Policy::default()
    };
    (manifest, key, trust, policy)
}

fn envelope(manifest: &Manifest, key: &SigningKey) -> Vec<u8> {
    serde_json::to_vec(&SignedManifest::sign(manifest, "test", key).unwrap()).unwrap()
}

#[test]
fn budget_fit_matches_each_runtime_admission_boundary() {
    let (mut manifest, key, trust, policy) = fixture();
    assert!(
        manifest
            .limits
            .exceeded_fields(&policy.max_limits)
            .unwrap()
            .is_empty()
    );
    for field in [
        "fuel",
        "memory_bytes",
        "timeout_ms",
        "input_bytes",
        "output_bytes",
    ] {
        manifest.limits = policy.max_limits.clone();
        match field {
            "fuel" => manifest.limits.fuel += 1,
            "memory_bytes" => manifest.limits.memory_bytes += 1,
            "timeout_ms" => manifest.limits.timeout_ms += 1,
            "input_bytes" => manifest.limits.input_bytes += 1,
            _ => manifest.limits.output_bytes += 1,
        }
        assert_eq!(
            manifest.limits.exceeded_fields(&policy.max_limits).unwrap(),
            vec![field]
        );
        assert!(matches!(
            VerifiedPackage::verify(
                &envelope(&manifest, &key),
                b"component".to_vec(),
                &trust,
                &policy
            ),
            Err(Error::Policy)
        ));
    }
    manifest.limits.fuel = 0;
    assert!(manifest.limits.exceeded_fields(&policy.max_limits).is_err());
}

#[test]
fn policy_authoring_validation_rejects_unusable_and_duplicate_grants() {
    assert!(Policy::default().validate().is_ok());
    let (_, _, _, mut policy) = fixture();
    assert!(policy.validate().is_ok());
    for names in [
        vec!["".into()],
        vec!["Bad/name".into()],
        vec!["x".repeat(65)],
        vec!["echo".into(), "echo".into()],
    ] {
        policy.allowed_tools = names;
        assert!(policy.validate().is_err());
    }
    policy = Policy::default();
    policy.max_component_bytes = 0;
    assert!(policy.validate().is_err());
    policy = Policy::default();
    policy.max_limits.fuel = 0;
    assert!(policy.validate().is_err());
}

#[test]
fn verifies_signature_digest_and_host_admission() {
    let (manifest, key, trust, policy) = fixture();
    let package = VerifiedPackage::verify(
        &envelope(&manifest, &key),
        b"component".to_vec(),
        &trust,
        &policy,
    )
    .unwrap();
    assert_eq!(package.manifest(), &manifest);
    assert_eq!(package.signer(), "test");
}

#[test]
fn tampering_and_unknown_signers_fail() {
    let (manifest, key, trust, policy) = fixture();
    let bytes = envelope(&manifest, &key);
    assert!(matches!(
        VerifiedPackage::verify(&bytes, b"changed".to_vec(), &trust, &policy),
        Err(Error::Digest)
    ));
    assert!(matches!(
        VerifiedPackage::verify(&bytes, b"component".to_vec(), &TrustStore::new(), &policy),
        Err(Error::Signature)
    ));
    let wrong = [(
        "test".into(),
        hex::encode(SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes()),
    )]
    .into();
    assert!(matches!(
        VerifiedPackage::verify(&bytes, b"component".to_vec(), &wrong, &policy),
        Err(Error::Signature)
    ));
}

#[test]
fn every_unsupported_capability_fails_closed() {
    let (manifest, key, trust, policy) = fixture();
    for field in [
        "filesystem",
        "network",
        "secrets",
        "subprocess",
        "clocks",
        "randomness",
    ] {
        let mut value = serde_json::to_value(&manifest).unwrap();
        value["capabilities"][field] = match field {
            "subprocess" => true.into(),
            "clocks" | "randomness" => "host".into(),
            _ => serde_json::json!(["anything"]),
        };
        let changed = serde_json::from_value(value).unwrap();
        assert!(
            matches!(
                VerifiedPackage::verify(
                    &envelope(&changed, &key),
                    b"component".to_vec(),
                    &trust,
                    &policy
                ),
                Err(Error::Policy)
            ),
            "{field}"
        );
    }
}

#[test]
fn malformed_versions_unknown_fields_and_budgets_are_rejected() {
    let (mut manifest, key, trust, policy) = fixture();
    manifest.limits.fuel += 1;
    assert!(matches!(
        VerifiedPackage::verify(
            &envelope(&manifest, &key),
            b"component".to_vec(),
            &trust,
            &policy
        ),
        Err(Error::Policy)
    ));
    manifest.limits.fuel = 0;
    assert!(matches!(
        VerifiedPackage::verify(
            &envelope(&manifest, &key),
            b"component".to_vec(),
            &trust,
            &policy
        ),
        Err(Error::Policy)
    ));
    let mut value = serde_json::to_value(&manifest).unwrap();
    value["surprise"] = true.into();
    assert!(serde_json::from_value::<Manifest>(value).is_err());
    manifest.schema_version = 2;
    assert!(SignedManifest::sign(&manifest, "test", &key).is_err());
}

#[test]
fn default_policy_denies_all_tools_and_size_is_bounded() {
    let (manifest, key, trust, mut policy) = fixture();
    assert!(matches!(
        VerifiedPackage::verify(
            &envelope(&manifest, &key),
            b"component".to_vec(),
            &trust,
            &Policy::default()
        ),
        Err(Error::Policy)
    ));
    policy.max_component_bytes = 1;
    assert!(matches!(
        VerifiedPackage::verify(
            &envelope(&manifest, &key),
            b"component".to_vec(),
            &trust,
            &policy
        ),
        Err(Error::Policy)
    ));
}
