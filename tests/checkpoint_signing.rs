use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use toolcapsule::{TrustStore, audit::AuditCheckpoint, checkpoint_signing::SignedCheckpoint};

fn fixture() -> (AuditCheckpoint, SigningKey, TrustStore) {
    let checkpoint = AuditCheckpoint {
        schema_version: 1,
        entries: 2,
        checkpoint: "a".repeat(64),
    };
    let key = SigningKey::from_bytes(&[42; 32]);
    let trust = TrustStore::from([(
        "custodian".into(),
        hex::encode(key.verifying_key().to_bytes()),
    )]);
    (checkpoint, key, trust)
}

#[test]
fn authenticated_roundtrip_and_revocation() {
    let (checkpoint, key, trust) = fixture();
    let signed = SignedCheckpoint::sign(&checkpoint, "custodian", &key).unwrap();
    let bytes = serde_json::to_vec(&signed).unwrap();
    let verified = SignedCheckpoint::verify(&bytes, &trust).unwrap();
    assert_eq!(verified.entries, checkpoint.entries);
    assert_eq!(verified.checkpoint, checkpoint.checkpoint);
    assert!(SignedCheckpoint::verify(&bytes, &TrustStore::new()).is_err());
    let wrong = TrustStore::from([(
        "custodian".into(),
        hex::encode(SigningKey::from_bytes(&[43; 32]).verifying_key().to_bytes()),
    )]);
    assert!(SignedCheckpoint::verify(&bytes, &wrong).is_err());
}

#[test]
fn tampering_unknown_fields_and_wrong_domain_fail() {
    let (checkpoint, key, trust) = fixture();
    let signed = SignedCheckpoint::sign(&checkpoint, "custodian", &key).unwrap();
    for field in [
        "checkpoint",
        "signature",
        "key_id",
        "schema_version",
        "extra",
    ] {
        let mut value = serde_json::to_value(&signed).unwrap();
        value[field] = if field == "schema_version" {
            serde_json::json!(2)
        } else {
            serde_json::json!("bad")
        };
        assert!(SignedCheckpoint::verify(&serde_json::to_vec(&value).unwrap(), &trust).is_err());
    }
    let mut unsigned_domain = signed.clone();
    unsigned_domain.signature = hex::encode(
        key.sign(&STANDARD.decode(&signed.checkpoint).unwrap())
            .to_bytes(),
    );
    assert!(
        SignedCheckpoint::verify(&serde_json::to_vec(&unsigned_domain).unwrap(), &trust).is_err()
    );
}

#[test]
fn bounded_strict_payload_and_signer_input() {
    let (mut checkpoint, key, trust) = fixture();
    assert!(SignedCheckpoint::verify(&vec![b' '; 8193], &trust).is_err());
    assert!(SignedCheckpoint::sign(&checkpoint, "", &key).is_err());
    assert!(SignedCheckpoint::sign(&checkpoint, &"x".repeat(65), &key).is_err());
    checkpoint.entries = 0;
    assert!(SignedCheckpoint::sign(&checkpoint, "custodian", &key).is_err());
    for payload in [
        br#"{"schema_version":2,"entries":0,"checkpoint":""}"#.to_vec(),
        vec![b' '; 4097],
    ] {
        let signature = key.sign(
            &[
                b"toolcapsule-audit-checkpoint-v1\0".as_slice(),
                payload.as_slice(),
            ]
            .concat(),
        );
        let envelope = SignedCheckpoint {
            schema_version: 1,
            key_id: "custodian".into(),
            checkpoint: STANDARD.encode(&payload),
            signature: hex::encode(signature.to_bytes()),
        };
        assert!(SignedCheckpoint::verify(&serde_json::to_vec(&envelope).unwrap(), &trust).is_err());
    }
}
