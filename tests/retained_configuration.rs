use serde_json::{Value, json};
use std::{path::Path, process::Command};
use toolcapsule::configuration::{MAX_RESULT_BYTES, read_result, verify_result};

fn fixture() -> Value {
    json!({"schema_version":1,"kind":"configuration","valid":true,
      "checks":{"limits":{"status":"valid"},"policy":{"status":"valid","denies_all_tools":true},
      "trust":{"status":"valid","denies_all_signers":true}},
      "budget_fit":{"status":"fits","exceeded_fields":[]},"package_admission":"not_checked"})
}

#[test]
fn accepts_all_check_combinations_and_rejects_contradictions() {
    for mask in 0..8 {
        let mut report = fixture();
        for (bit, name) in ["limits", "policy", "trust"].iter().enumerate() {
            if mask & (1 << bit) != 0 {
                report["checks"][name] = json!({"status":"invalid","error":"invalid_format"});
            }
        }
        report["valid"] = json!(mask == 0);
        report["budget_fit"]["status"] = json!(if mask & 3 == 0 { "fits" } else { "not_checked" });
        assert!(read_result(&serde_json::to_vec(&report).unwrap()).is_ok());
        report["valid"] = json!(mask != 0);
        assert!(read_result(&serde_json::to_vec(&report).unwrap()).is_err());
    }
    for (pointer, value) in [
        ("/schema_version", json!(1.0)),
        ("/schema_version", json!(true)),
        ("/schema_version", json!(2)),
        ("/kind", json!("limits")),
        ("/package_admission", json!("admitted")),
        ("/budget_fit/status", json!("not_checked")),
        ("/budget_fit/status", json!("exceeds")),
        ("/budget_fit/exceeded_fields", json!(["fuel"])),
        ("/checks/policy", json!({"status":"valid"})),
        (
            "/checks/trust",
            json!({"status":"invalid","error":"io_error","denies_all_signers":true}),
        ),
        ("/checks/limits", json!({"status":"valid","unknown":0})),
    ] {
        let mut report = fixture();
        *report.pointer_mut(pointer).unwrap() = value;
        assert!(
            read_result(&serde_json::to_vec(&report).unwrap()).is_err(),
            "{pointer}"
        );
    }
    for fields in [
        json!(["fuel"]),
        json!([
            "fuel",
            "memory_bytes",
            "timeout_ms",
            "input_bytes",
            "output_bytes"
        ]),
    ] {
        let mut report = fixture();
        report["budget_fit"] = json!({"status":"exceeds","exceeded_fields":fields});
        assert!(read_result(&serde_json::to_vec(&report).unwrap()).is_ok());
    }
    for fields in [
        json!(["fuel", "fuel"]),
        json!(["output_bytes", "fuel"]),
        json!(["unknown"]),
    ] {
        let mut report = fixture();
        report["budget_fit"] = json!({"status":"exceeds","exceeded_fields":fields});
        assert!(read_result(&serde_json::to_vec(&report).unwrap()).is_err());
    }
    for code in [
        "invalid_format",
        "invalid_configuration",
        "input_too_large",
        "io_error",
        "validation_failed",
    ] {
        let mut report = fixture();
        report["valid"] = json!(false);
        report["checks"]["trust"] = json!({"status":"invalid","error":code});
        assert!(read_result(&serde_json::to_vec(&report).unwrap()).is_ok());
    }
}

#[test]
fn rejects_ambiguous_malformed_and_oversized_evidence() {
    let raw = fixture().to_string();
    for bad in [
        raw.replacen("{", "{\"schema_version\":1,", 1),
        raw.replace(
            "\"status\":\"valid\"",
            "\"status\":\"valid\",\"status\":\"valid\"",
        ),
        raw.clone() + "{}",
        "[]".into(),
        "{".into(),
    ] {
        assert!(read_result(bad.as_bytes()).is_err());
    }
    assert!(read_result(&[0xff]).is_err());
    assert!(read_result(&vec![b' '; MAX_RESULT_BYTES + 1]).is_err());
}

fn call(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn cli_checks_and_source_verification_preserve_inputs_and_sanitize_errors() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for (command, file) in [
        ("limits-template", "limits.json"),
        ("policy-template", "policy.json"),
    ] {
        assert!(call(root, &[command, "--out", file]).status.success());
    }
    std::fs::write(root.join("trust.json"), b"{}").unwrap();
    let output = call(
        root,
        &[
            "configuration-check",
            "--limits",
            "limits.json",
            "--policy",
            "policy.json",
            "--trust",
            "trust.json",
        ],
    );
    std::fs::write(root.join("result.json"), &output.stdout).unwrap();
    assert!(
        call(
            root,
            &["configuration-result-check", "--report", "result.json"]
        )
        .status
        .success()
    );
    let args = [
        "configuration-result-verify",
        "--report",
        "result.json",
        "--limits",
        "limits.json",
        "--policy",
        "policy.json",
        "--trust",
        "trust.json",
    ];
    assert!(call(root, &args).status.success());
    let mut altered = fixture();
    altered["checks"]["trust"]["denies_all_signers"] = json!(false);
    std::fs::write(root.join("result.json"), altered.to_string()).unwrap();
    let output = call(root, &args);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stderr).unwrap(),
        json!({"schema_version":1,"kind":"configuration_result","error":"invalid_format"})
    );
    assert_eq!(
        std::fs::read(root.join("result.json")).unwrap(),
        altered.to_string().as_bytes()
    );
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 4);
    let output = call(
        root,
        &["configuration-result-check", "--report", "private_marker"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private_marker"));
    // Matching failure summaries remain evidence, not successful configuration.
    let (failed, _) = toolcapsule::configuration::check(
        &root.join("missing"),
        &root.join("policy.json"),
        &root.join("trust.json"),
    );
    assert!(
        verify_result(
            failed.to_string().as_bytes(),
            &root.join("missing"),
            &root.join("policy.json"),
            &root.join("trust.json")
        )
        .is_ok()
    );
}
