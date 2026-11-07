use serde_json::{Value, json};
use toolcapsule::configuration_diff::{FIELDS, compare, encode, schema};
fn fixture() -> Value {
    json!({"schema_version":1,"kind":"configuration","valid":true,
"checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},
"budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"})
}
#[test]
fn directional_and_unchanged_summaries() {
    let before = fixture();
    let mut after = before.clone();
    after["checks"]["policy"]["denies_all_tools"] = json!(false);
    let a = before.to_string();
    let b = after.to_string();
    let diff = compare(a.as_bytes(), b.as_bytes()).unwrap();
    assert_eq!(
        diff["changed_fields"],
        json!(["/checks/policy/denies_all_tools"])
    );
    let reverse = compare(b.as_bytes(), a.as_bytes()).unwrap();
    assert_eq!(diff["before"], reverse["after"]);
    assert_eq!(diff["changed_fields"], reverse["changed_fields"]);
    let unchanged = compare(a.as_bytes(), &serde_json::to_vec_pretty(&before).unwrap()).unwrap();
    assert_eq!(unchanged["identical_summaries"], true);
    assert_eq!(unchanged["changed_fields"], json!([]));
    assert!(encode(a.as_bytes(), b.as_bytes()).unwrap().ends_with(b"\n"));
}
#[test]
fn failed_summaries_and_schema_contract() {
    let before = fixture();
    let mut after = before.clone();
    after["valid"] = json!(false);
    for name in ["limits", "policy", "trust"] {
        after["checks"][name] = json!({"status":"invalid","error":"invalid_format"});
    }
    after["budget_fit"]["status"] = json!("not_checked");
    let diff = compare(before.to_string().as_bytes(), after.to_string().as_bytes()).unwrap();
    assert_eq!(diff["changed_fields"].as_array().unwrap().len(), 10);
    let mut exceeded = before.clone();
    exceeded["budget_fit"] = json!({"status":"exceeds","exceeded_fields":["fuel"]});
    let budget = compare(
        before.to_string().as_bytes(),
        exceeded.to_string().as_bytes(),
    )
    .unwrap();
    assert_eq!(
        budget["changed_fields"],
        json!(["/budget_fit/status", "/budget_fit/exceeded_fields"])
    );
    let shape = schema();
    assert_eq!(shape["additionalProperties"], false);
    assert_eq!(
        shape["properties"]["changed_fields"]["items"]["enum"],
        json!(FIELDS)
    );
    let validation: Value =
        serde_json::from_str(include_str!("../schemas/validation.schema.json")).unwrap();
    assert_eq!(shape["$defs"]["summary"], validation["oneOf"][5]);
}
#[test]
fn cli_exclusive_output_and_failure_atomicity() {
    use std::process::Command;
    let dir = tempfile::tempdir().unwrap();
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    let out = dir.path().join("diff.json");
    std::fs::write(&before, fixture().to_string()).unwrap();
    std::fs::write(&after, fixture().to_string()).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .arg("configuration-diff")
            .arg(&before)
            .arg(&after)
            .arg("--out")
            .arg(&out)
            .output()
            .unwrap()
    };
    let stdout = Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .arg("configuration-diff")
        .arg(&before)
        .arg(&after)
        .output()
        .unwrap();
    assert!(stdout.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&stdout.stdout).unwrap()["identical_summaries"],
        true
    );
    assert!(run().status.success());
    let saved = std::fs::read(&out).unwrap();
    assert!(!run().status.success());
    assert_eq!(saved, std::fs::read(&out).unwrap());
    for bad in [b"{}".to_vec(), vec![b' '; 4097], b"[]".to_vec()] {
        std::fs::write(&after, &bad).unwrap();
        assert!(!run().status.success());
        assert_eq!(saved, std::fs::read(&out).unwrap());
        assert!(compare(fixture().to_string().as_bytes(), &bad).is_err());
        assert!(compare(&bad, fixture().to_string().as_bytes()).is_err());
        let absent = dir.path().join("must-not-exist.json");
        let failure = Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .arg("configuration-diff")
            .arg(&before)
            .arg(&after)
            .arg("--out")
            .arg(&absent)
            .output()
            .unwrap();
        assert!(!failure.status.success());
        assert!(!absent.exists());
        assert!(
            !String::from_utf8_lossy(&failure.stderr)
                .contains(&dir.path().to_string_lossy().to_string())
        );
        assert_eq!(
            std::fs::read(&before).unwrap(),
            fixture().to_string().as_bytes()
        );
    }
}
