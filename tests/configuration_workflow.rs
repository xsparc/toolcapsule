use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Command, Output},
};

fn call(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn json_errors_have_fixed_codes_and_preserve_exit_behavior() {
    let dir = tempfile::tempdir().unwrap();
    for (command, flag, kind) in [
        ("limits-check", "--limits", "limits"),
        ("policy-check", "--policy", "policy"),
        ("trust-check", "--trust", "trust"),
        ("budget-fit", "--limits", "budget_fit"),
    ] {
        for (bytes, code) in [
            (None, "io_error"),
            (Some(b"private_marker".to_vec()), "invalid_format"),
            (Some(vec![b' '; 65537]), "input_too_large"),
        ] {
            let path = dir.path().join("private_marker.json");
            if path.exists() {
                std::fs::remove_file(&path).unwrap();
            }
            if let Some(bytes) = &bytes {
                std::fs::write(&path, bytes).unwrap();
            }
            let mut args = vec![command, flag, "private_marker.json", "--json-errors"];
            if command == "budget-fit" {
                args.extend(["--policy", "unused.json"]);
            }
            let result = call(dir.path(), &args);
            assert_eq!(result.status.code(), Some(2));
            assert!(result.stdout.is_empty());
            assert_eq!(
                serde_json::from_slice::<Value>(&result.stderr).unwrap(),
                json!({"schema_version": 1, "kind": kind, "error": code})
            );
            if let Some(bytes) = bytes {
                assert_eq!(std::fs::read(&path).unwrap(), bytes);
            }
        }
    }
    let limits = toolcapsule::Limits {
        fuel: 0,
        ..Default::default()
    };
    std::fs::write(
        dir.path().join("limits.json"),
        serde_json::to_vec(&limits).unwrap(),
    )
    .unwrap();
    let result = call(
        dir.path(),
        &["limits-check", "--limits", "limits.json", "--json-errors"],
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stderr).unwrap()["error"],
        "invalid_configuration"
    );
    let default = call(dir.path(), &["limits-check", "--limits", "limits.json"]);
    assert_eq!(default.status.code(), Some(2));
    assert_eq!(
        default.stderr,
        b"toolcapsule: capability or budget denied by host policy\n"
    );
}

#[test]
fn configuration_authoring_workflow_proves_explicit_denials() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy("target/fixtures/guest.wasm", dir.path().join("guest.wasm")).unwrap();
    for args in [
        vec!["limits-template", "--out", "limits.json"],
        vec![
            "policy-template",
            "--limits",
            "limits.json",
            "--out",
            "policy.json",
        ],
        vec![
            "keygen",
            "--key",
            "synthetic.key",
            "--trust",
            "trust.json",
            "--key-id",
            "fixture",
        ],
        vec!["limits-check", "--limits", "limits.json", "--json-errors"],
        vec!["policy-check", "--policy", "policy.json", "--json-errors"],
        vec!["trust-check", "--trust", "trust.json", "--json-errors"],
        vec![
            "budget-fit",
            "--limits",
            "limits.json",
            "--policy",
            "policy.json",
            "--json-errors",
        ],
        vec![
            "pack",
            "--component",
            "guest.wasm",
            "--name",
            "fixture",
            "--limits",
            "limits.json",
            "--key",
            "synthetic.key",
            "--key-id",
            "fixture",
            "--out",
            "signed.json",
        ],
    ] {
        let result = call(dir.path(), &args);
        assert!(result.status.success(), "command failed: {}", args[0]);
        assert!(result.stderr.is_empty());
    }
    let package = [
        "--component",
        "guest.wasm",
        "--manifest",
        "signed.json",
        "--trust",
        "trust.json",
        "--policy",
        "policy.json",
    ];
    let invoke = |command| call(dir.path(), &[&[command][..], &package].concat());
    assert_eq!(invoke("verify").status.code(), Some(2));
    let mut policy: Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("policy.json")).unwrap()).unwrap();
    policy["allowed_tools"] = json!(["fixture"]);
    std::fs::write(
        dir.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    assert!(invoke("verify").status.success());
    let result = invoke("preflight");
    assert!(result.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["executed"],
        false
    );
    policy["max_limits"]["fuel"] = json!(1);
    std::fs::write(
        dir.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    let result = call(
        dir.path(),
        &[
            "budget-fit",
            "--limits",
            "limits.json",
            "--policy",
            "policy.json",
            "--json-errors",
        ],
    );
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["exceeded_fields"],
        json!(["fuel"])
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stderr).unwrap()["error"],
        "budget_exceeded"
    );
    let default = call(
        dir.path(),
        &[
            "budget-fit",
            "--limits",
            "limits.json",
            "--policy",
            "policy.json",
        ],
    );
    assert_eq!(default.status.code(), Some(1));
    assert!(default.stderr.is_empty());
    assert_eq!(invoke("verify").status.code(), Some(2));
    assert_eq!(invoke("preflight").status.code(), Some(2));
}
