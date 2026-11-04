use serde_json::{Value, json};
use std::{path::Path, process::Command};

fn call(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn independent_checks_are_read_only_and_never_admit_packages() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for (cmd, filename) in [
        ("limits-template", "limits.json"),
        ("policy-template", "policy.json"),
    ] {
        assert!(call(root, &[cmd, "--out", filename]).status.success());
    }
    std::fs::write(root.join("trust.json"), b"{}").unwrap();
    let args = [
        "configuration-check",
        "--limits",
        "limits.json",
        "--policy",
        "policy.json",
        "--trust",
        "trust.json",
    ];
    let original = std::fs::read(root.join("limits.json")).unwrap();
    let output = call(root, &args);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report,
        json!({"schema_version":1,"kind":"configuration","valid":true,
        "checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},"trust":{"status":"valid","denies_all_signers":true}},
        "budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"})
    );
    let mut limits: Value = serde_json::from_slice(&original).unwrap();
    limits["fuel"] = json!(limits["fuel"].as_u64().unwrap() + 1);
    std::fs::write(
        root.join("limits.json"),
        serde_json::to_vec(&limits).unwrap(),
    )
    .unwrap();
    let output = call(root, &args);
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["budget_fit"],
        json!({"status":"exceeds","exceeded_fields":["fuel"]})
    );
    std::fs::write(root.join("trust.json"), b"private_marker").unwrap();
    let output = call(root, &args);
    assert_eq!(output.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["budget_fit"]["status"], "exceeds");
    assert_eq!(report["checks"]["trust"]["error"], "invalid_format");
    std::fs::remove_file(root.join("limits.json")).unwrap();
    std::fs::write(root.join("policy.json"), vec![b' '; 65537]).unwrap();
    let output = call(root, &args);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("private_marker"));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["checks"]["limits"]["error"], "io_error");
    assert_eq!(report["checks"]["policy"]["error"], "input_too_large");
    assert_eq!(report["budget_fit"]["status"], "not_checked");
    assert_eq!(
        std::fs::read(root.join("trust.json")).unwrap(),
        b"private_marker"
    );
    assert_eq!(
        std::fs::read(root.join("policy.json")).unwrap().len(),
        65537
    );
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 2);
    assert_eq!(call(root, &["configuration-check"]).status.code(), Some(2));
}

#[test]
fn schema_export_and_cli_discriminators_stay_in_sync() {
    let dir = tempfile::tempdir().unwrap();
    let output = call(dir.path(), &["validation-schema"]);
    assert!(output.status.success());
    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        schema,
        serde_json::from_str::<Value>(include_str!("../schemas/validation.schema.json")).unwrap()
    );
    for (command, flag, kind) in [
        ("limits-check", "--limits", "limits"),
        ("policy-check", "--policy", "policy"),
        ("trust-check", "--trust", "trust"),
    ] {
        let output = call(dir.path(), &[command, flag, "absent.json", "--json-errors"]);
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        let shape = &schema["oneOf"][0];
        assert_eq!(
            error.as_object().unwrap().len(),
            shape["required"].as_array().unwrap().len()
        );
        assert_eq!(error["kind"], kind);
        assert!(
            shape["properties"]["kind"]["enum"]
                .as_array()
                .unwrap()
                .contains(&error["kind"])
        );
        assert!(
            shape["properties"]["error"]["enum"]
                .as_array()
                .unwrap()
                .contains(&error["error"])
        );
    }
    for shape in schema["oneOf"].as_array().unwrap() {
        assert_eq!(shape["additionalProperties"], false);
        assert_eq!(
            shape["properties"].as_object().unwrap().len(),
            shape["required"].as_array().unwrap().len()
        );
    }
    for (command, filename) in [
        ("limits-template", "limits.json"),
        ("policy-template", "policy.json"),
    ] {
        assert!(
            call(dir.path(), &[command, "--out", filename])
                .status
                .success()
        );
    }
    std::fs::write(dir.path().join("trust.json"), b"{}").unwrap();
    for (index, args) in [
        (1, vec!["limits-check", "--limits", "limits.json"]),
        (2, vec!["policy-check", "--policy", "policy.json"]),
        (3, vec!["trust-check", "--trust", "trust.json"]),
        (
            4,
            vec![
                "budget-fit",
                "--limits",
                "limits.json",
                "--policy",
                "policy.json",
            ],
        ),
        (
            5,
            vec![
                "configuration-check",
                "--limits",
                "limits.json",
                "--policy",
                "policy.json",
                "--trust",
                "trust.json",
            ],
        ),
    ] {
        let output = call(dir.path(), &args);
        assert!(output.status.success());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let shape = &schema["oneOf"][index];
        assert_eq!(value["kind"], shape["properties"]["kind"]["const"]);
        assert_eq!(
            value["schema_version"],
            shape["properties"]["schema_version"]["const"]
        );
        let keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
        let mut required: Vec<_> = shape["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        required.sort();
        assert_eq!(keys, required);
    }
    assert_eq!(schema["examples"].as_array().unwrap().len(), 6);
}
