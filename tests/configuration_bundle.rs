use serde_json::json;
use std::process::Command;
use toolcapsule::configuration_bundle::{
    MAX_BUNDLE_BYTES, create, encode, read_bundle, schema, verify_sources,
};
fn pair() -> (Vec<u8>, Vec<u8>) {
    let a = json!({"schema_version":1,"kind":"configuration","valid":true,"checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},"budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"});
    let mut b = a.clone();
    b["budget_fit"] = json!({"status":"exceeds","exceeded_fields":["fuel","memory_bytes"]});
    (
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap(),
    )
}

#[test]
fn closed_bundle_roundtrip_and_independent_sources() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let value = read_bundle(&raw).unwrap();
    assert_eq!(value, create(&a, &b).unwrap());
    verify_sources(&raw, &a, &b).unwrap();
    assert!(verify_sources(&raw, &b, &a).is_err());
    let replacement = encode(&a, &a).unwrap();
    read_bundle(&replacement).unwrap();
    assert!(verify_sources(&replacement, &a, &b).is_err());
    assert_eq!(schema()["additionalProperties"], false);
    for (key, v) in [
        ("schema_version", json!(true)),
        ("schema_version", json!(1.0)),
        ("schema_version", json!(2)),
        ("extra", json!(1)),
        ("interpretation", json!("trusted")),
        ("before", json!({})),
    ] {
        let mut bad = value.clone();
        bad[key] = v;
        assert!(read_bundle(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for key in ["before", "after", "difference"] {
        let mut bad = value.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(read_bundle(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = value.clone();
    bad["difference"]["before"] = bad["after"].clone();
    assert!(read_bundle(&serde_json::to_vec(&bad).unwrap()).is_err());
    let dup = String::from_utf8(raw.clone()).unwrap().replacen(
        "\"schema_version\": 1",
        "\"schema_version\": 1, \"schema_version\": 1",
        1,
    );
    assert!(read_bundle(dup.as_bytes()).is_err());
    assert!(read_bundle(&vec![b' '; MAX_BUNDLE_BYTES + 1]).is_err());
    assert!(encode(&vec![b' '; 4097], &b).is_err());
    let mut failed: serde_json::Value = serde_json::from_slice(&a).unwrap();
    failed["valid"] = json!(false);
    failed["checks"]["limits"] = json!({"status":"invalid","error":"invalid_format"});
    failed["budget_fit"] = json!({"status":"not_checked","exceeded_fields":[]});
    read_bundle(&encode(&serde_json::to_vec(&failed).unwrap(), &b).unwrap()).unwrap();
}
#[test]
fn cli_exports_exclusively_and_verifies_without_writes() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    std::fs::write(d.path().join("a"), &a).unwrap();
    std::fs::write(d.path().join("b"), &b).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(
        run(&["configuration-bundle", "a", "b", "--out", "bundle"])
            .status
            .success()
    );
    let raw = std::fs::read(d.path().join("bundle")).unwrap();
    assert!(
        !run(&["configuration-bundle", "a", "b", "--out", "bundle"])
            .status
            .success()
    );
    assert!(
        run(&["configuration-bundle-verify", "bundle"])
            .status
            .success()
    );
    assert!(
        run(&[
            "configuration-bundle-verify",
            "bundle",
            "--before",
            "a",
            "--after",
            "b"
        ])
        .status
        .success()
    );
    assert!(
        !run(&["configuration-bundle-verify", "bundle", "--before", "a"])
            .status
            .success()
    );
    assert!(
        !run(&[
            "configuration-bundle-verify",
            "bundle",
            "--before",
            "b",
            "--after",
            "a"
        ])
        .status
        .success()
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&run(&["configuration-bundle-schema"]).stdout)
            .unwrap(),
        schema()
    );
    assert_eq!(std::fs::read(d.path().join("bundle")).unwrap(), raw);
    assert_eq!(std::fs::read(d.path().join("a")).unwrap(), a);
    std::fs::write(d.path().join("bad"), b"{}").unwrap();
    assert!(
        !run(&["configuration-bundle", "bad", "b", "--out", "blocked"])
            .status
            .success()
    );
    assert!(!d.path().join("blocked").exists());
}
