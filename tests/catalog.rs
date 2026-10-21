use ed25519_dalek::SigningKey;
use serde_json::json;
use toolcapsule::{catalog::prepare_catalog, package::digest, *};

#[test]
fn aggregate_bytes_and_policy_are_checked_before_compilation() {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[7; 32]);
    let trust: TrustStore = [("test".into(), hex::encode(key.verifying_key().to_bytes()))].into();
    // Deliberately not Wasm: reaching compilation would return Component, not InputLimit.
    let bytes = vec![0; 8 * 1024 * 1024];
    std::fs::write(dir.path().join("source.wasm"), &bytes).unwrap();
    let mut entries = Vec::new();
    let mut policy = Policy::default();
    for i in 0..9 {
        let name = format!("tool-{i}");
        policy.allowed_tools.push(name.clone());
        let manifest = Manifest {
            schema_version: 1,
            name,
            version: "1".into(),
            description: "Synthetic admission fixture".into(),
            abi: "wasi:cli/command@0.2".into(),
            component_sha256: digest(&bytes),
            capabilities: Capabilities::default(),
            limits: Limits::default(),
        };
        let filename = format!("signed-{i}.json");
        std::fs::write(
            dir.path().join(&filename),
            serde_json::to_vec(&SignedManifest::sign(&manifest, "test", &key).unwrap()).unwrap(),
        )
        .unwrap();
        entries.push(json!({"manifest":filename,"component":"source.wasm"}));
    }
    let path = dir.path().join("catalog.json");
    std::fs::write(
        &path,
        json!({"schema_version":1,"packages":entries}).to_string(),
    )
    .unwrap();
    assert!(matches!(
        prepare_catalog(&path, &trust, &policy),
        Err(Error::InputLimit)
    ));
    policy.allowed_tools.clear();
    assert!(matches!(
        prepare_catalog(&path, &trust, &policy),
        Err(Error::Policy)
    ));
    policy.allowed_tools.push("tool-0".into());
    std::fs::write(dir.path().join("source.wasm"), b"changed").unwrap();
    assert!(matches!(
        prepare_catalog(&path, &trust, &policy),
        Err(Error::Digest)
    ));
}
