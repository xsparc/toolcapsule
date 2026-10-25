use ed25519_dalek::SigningKey;
use toolcapsule::{
    package::digest,
    runtime::{PROFILE, Runtime},
    *,
};

fn admitted(bytes: Vec<u8>) -> VerifiedPackage {
    let key = SigningKey::from_bytes(&[19; 32]);
    let manifest = Manifest {
        schema_version: 1,
        name: "fixture".into(),
        version: "1".into(),
        description: "Synthetic preflight".into(),
        abi: "wasi:cli/command@0.2".into(),
        component_sha256: digest(&bytes),
        capabilities: Capabilities::default(),
        limits: Limits::default(),
    };
    let envelope =
        serde_json::to_vec(&SignedManifest::sign(&manifest, "test", &key).unwrap()).unwrap();
    let trust = TrustStore::from([("test".into(), hex::encode(key.verifying_key().to_bytes()))]);
    let policy = Policy {
        allowed_tools: vec!["fixture".into()],
        ..Policy::default()
    };
    VerifiedPackage::verify(&envelope, bytes, &trust, &policy).unwrap()
}

#[test]
fn real_command_has_profile_and_admitted_identity() {
    let bytes = std::fs::read("target/fixtures/guest.wasm").unwrap();
    let expected = digest(&bytes);
    let result = Runtime::new().unwrap().preflight(admitted(bytes)).unwrap();
    assert_eq!(result.schema_version, 1);
    assert_eq!(result.profile, PROFILE);
    assert_eq!(result.manifest.component_sha256, expected);
    assert_eq!(result.signer, "test");
    assert!(result.compiled);
    assert!(!result.executed);
}

#[test]
fn rejects_bad_binary_missing_command_and_unresolved_import() {
    let runtime = Runtime::new().unwrap();
    for bytes in [
        b"not wasm".to_vec(),
        wat::parse_str("(component)").unwrap(),
        wat::parse_str("(component (import \"unavailable\" (func)))").unwrap(),
    ] {
        assert!(matches!(
            runtime.preflight(admitted(bytes)),
            Err(Error::Component)
        ));
    }
}

#[test]
fn preparation_does_not_execute_core_start_or_command() {
    let bytes = wat::parse_str(
        r#"(component
        (core module $m
            (func $start unreachable) (start $start)
            (func (export "run") (result i32) unreachable))
        (core instance $i (instantiate $m))
        (func $run (result (result)) (canon lift (core func $i "run")))
        (instance $cli (export "run" (func $run)))
        (export "wasi:cli/run@0.2.0" (instance $cli)))"#,
    )
    .unwrap();
    assert!(
        !Runtime::new()
            .unwrap()
            .preflight(admitted(bytes))
            .unwrap()
            .executed
    );
}
