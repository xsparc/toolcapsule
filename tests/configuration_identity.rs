use serde_json::json;
use std::process::Command;
use toolcapsule::{
    configuration_diff::encode,
    configuration_identity::{identity, verify_expected},
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
fn normalized_identity_and_strict_gate() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let compact = serde_json::to_vec(&value).unwrap();
    let record = identity(&raw).unwrap();
    assert_eq!(record, identity(&compact).unwrap());
    let digest = record["difference_sha256"].as_str().unwrap();
    assert_eq!(
        digest,
        "c00bfd5c0d35b97ee70a0e48903f530c60ce8e2288eea36aed655ac8df4b8272"
    );
    let reordered = format!(
        "{{{}}}",
        value
            .as_object()
            .unwrap()
            .iter()
            .rev()
            .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), v))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(record, identity(reordered.as_bytes()).unwrap());
    verify_expected(&raw, digest).unwrap();
    assert_ne!(record, identity(&encode(&b, &a).unwrap()).unwrap());
    assert_ne!(record, identity(&encode(&a, &a).unwrap()).unwrap());
    for bad in [
        "".into(),
        "0".repeat(64),
        digest.to_uppercase(),
        format!(" {digest}"),
        "g".repeat(64),
    ] {
        assert!(verify_expected(&raw, &bad).is_err());
    }
    assert!(identity(b"{}").is_err());
    assert!(identity(&vec![b' '; 16385]).is_err());
}
#[test]
fn cli_gates_check_export_and_source_verification() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    for (name, data) in [
        ("diff.json", raw.clone()),
        ("a.json", a.clone()),
        ("b.json", b.clone()),
    ] {
        std::fs::write(d.path().join(name), data).unwrap();
    }
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    let id = run(&["configuration-diff-identity", "diff.json"]);
    assert!(id.status.success());
    let id: serde_json::Value = serde_json::from_slice(&id.stdout).unwrap();
    let digest = id["difference_sha256"].as_str().unwrap();
    let wrong = "0".repeat(64);
    for cmd in ["configuration-diff-html", "configuration-diff-csv"] {
        let out = if cmd.ends_with("html") {
            "review.html"
        } else {
            "review.csv"
        };
        assert!(
            !run(&[cmd, "diff.json", "--out", out, "--expected-sha256", &wrong])
                .status
                .success()
        );
        assert!(!d.path().join(out).exists());
        assert!(
            !run(&[
                cmd,
                "diff.json",
                "--out",
                out,
                "--expected-sha256",
                digest,
                "--before",
                "b.json",
                "--after",
                "a.json"
            ])
            .status
            .success()
        );
        assert!(!d.path().join(out).exists());
        assert!(
            run(&[
                cmd,
                "diff.json",
                "--out",
                out,
                "--expected-sha256",
                digest,
                "--before",
                "a.json",
                "--after",
                "b.json"
            ])
            .status
            .success()
        );
    }
    assert!(
        run(&[
            "configuration-diff-check",
            "diff.json",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    assert!(
        !run(&[
            "configuration-diff-check",
            "diff.json",
            "--expected-sha256",
            &wrong
        ])
        .status
        .success()
    );
    assert!(
        run(&[
            "configuration-diff-verify",
            "diff.json",
            "a.json",
            "b.json",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    assert!(
        !run(&[
            "configuration-diff-verify",
            "diff.json",
            "b.json",
            "a.json",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    assert!(
        run(&[
            "configuration-diff-csv-verify",
            "diff.json",
            "review.csv",
            "--expected-sha256",
            digest
        ])
        .status
        .success()
    );
    let failure = run(&[
        "configuration-diff-csv-verify",
        "diff.json",
        "review.csv",
        "--expected-sha256",
        &wrong,
    ]);
    assert!(!failure.status.success());
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("diff.json"));
    assert_eq!(std::fs::read(d.path().join("diff.json")).unwrap(), raw);
}
