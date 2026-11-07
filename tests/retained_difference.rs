use serde_json::{Value, json};
use std::process::Command;
use toolcapsule::configuration_diff::{
    MAX_DIFF_BYTES, compare, encode, read_difference, verify_difference,
};
fn summary() -> Value {
    json!({"schema_version":1,"kind":"configuration","valid":true,"checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},"budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"})
}
fn pair() -> (Vec<u8>, Vec<u8>) {
    let a = summary();
    let mut b = a.clone();
    b["checks"]["policy"]["denies_all_tools"] = json!(false);
    (
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap(),
    )
}
#[test]
fn roundtrip_sources_replacement_and_reverse() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    assert_eq!(read_difference(&raw).unwrap(), compare(&a, &b).unwrap());
    verify_difference(&raw, &a, &b).unwrap();
    verify_difference(&raw, &serde_json::to_vec_pretty(&summary()).unwrap(), &b).unwrap();
    for replacement in [encode(&a, &a).unwrap(), encode(&b, &a).unwrap()] {
        assert!(read_difference(&replacement).is_ok());
        assert!(verify_difference(&replacement, &a, &b).is_err());
    }
    let mut failed = summary();
    failed["valid"] = json!(false);
    failed["checks"]["trust"] = json!({"status":"invalid","error":"io_error"});
    assert!(read_difference(&encode(&a, &serde_json::to_vec(&failed).unwrap()).unwrap()).is_ok());
}
#[test]
fn reject_every_derived_field_and_nested_invalid_summary() {
    let (a, b) = pair();
    let valid = compare(&a, &b).unwrap();
    for (pointer, value) in [
        ("/schema_version", json!(true)),
        ("/schema_version", json!(1.0)),
        ("/schema_version", json!(2)),
        ("/kind", json!("configuration")),
        ("/direction", json!("after_to_before")),
        ("/identical_summaries", json!(true)),
        ("/interpretation", json!("authority")),
        ("/changed_fields", json!([])),
        ("/changed_fields", json!(["/unknown"])),
        (
            "/changed_fields",
            json!([
                "/checks/policy/denies_all_tools",
                "/checks/policy/denies_all_tools"
            ]),
        ),
        ("/before/valid", json!(false)),
        ("/after/checks/policy/denies_all_tools", json!(null)),
    ] {
        let mut bad = valid.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            read_difference(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{pointer}"
        );
    }
    for key in valid.as_object().unwrap().keys() {
        let mut bad = valid.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(read_difference(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let text = serde_json::to_string(&valid).unwrap();
    for bad in [
        text.replacen("{", "{\"schema_version\":1,", 1),
        text.replace(
            "\"status\":\"valid\"",
            "\"status\":\"valid\",\"status\":\"valid\"",
        ),
        text.replace(
            "\"denies_all_tools\":",
            "\"unknown\":0,\"denies_all_tools\":",
        ),
        text.clone() + "{}",
        "[]".into(),
        "[".repeat(2000),
    ] {
        assert!(read_difference(bad.as_bytes()).is_err());
    }
    assert!(read_difference(&vec![b' '; MAX_DIFF_BYTES + 1]).is_err());
    assert!(read_difference(b"\xff").is_err());
    let mut boundary = encode(&a, &b).unwrap();
    boundary.resize(MAX_DIFF_BYTES, b' ');
    assert!(read_difference(&boundary).is_ok());
}
#[test]
fn cli_verification_is_read_only_and_sanitized() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    let report = dir.path().join("report.json");
    std::fs::write(&before, &a).unwrap();
    std::fs::write(&after, &b).unwrap();
    let raw = encode(&a, &b).unwrap();
    std::fs::write(&report, &raw).unwrap();
    let run = |verify: bool| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_toolcapsule"));
        c.arg(if verify {
            "configuration-diff-verify"
        } else {
            "configuration-diff-check"
        })
        .arg(&report);
        if verify {
            c.arg(&before).arg(&after);
        }
        c.output().unwrap()
    };
    assert!(run(false).status.success());
    let success = run(true);
    assert!(success.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&success.stdout).unwrap()["verified"],
        true
    );
    std::fs::write(&report, encode(&b, &a).unwrap()).unwrap();
    assert!(run(false).status.success());
    let failure = run(true);
    assert!(!failure.status.success());
    assert!(failure.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("before.json"));
    for bad in [vec![b' '; MAX_DIFF_BYTES + 1], b"{}".to_vec()] {
        std::fs::write(&report, &bad).unwrap();
        assert!(!run(false).status.success());
        assert_eq!(std::fs::read(&report).unwrap(), bad);
    }
    assert_eq!(std::fs::read(&before).unwrap(), a);
    assert_eq!(std::fs::read(&after).unwrap(), b);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}
