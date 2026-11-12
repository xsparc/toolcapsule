use serde_json::json;
use std::process::Command;
use toolcapsule::{
    configuration_diff::encode,
    configuration_identity::{identity, read_identity, schema, verify_record},
    configuration_review::{render, verify},
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
fn strict_records_and_exact_html() {
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let record = identity(&raw).unwrap();
    let bytes = serde_json::to_vec(&record).unwrap();
    verify_record(&raw, &bytes).unwrap();
    let contract = schema();
    assert_eq!(contract["additionalProperties"], false);
    for (field, value) in [
        ("schema_version", json!(true)),
        ("schema_version", json!(2)),
        ("schema_version", json!(1.0)),
        ("encoding", json!("other")),
        ("difference_sha256", json!("A".repeat(64))),
        ("extra", json!(1)),
    ] {
        let mut bad = record.clone();
        bad[field] = value;
        assert!(read_identity(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let duplicate = format!(
        "{{\"schema_version\":1,{}",
        String::from_utf8(bytes.clone())
            .unwrap()
            .trim_start_matches('{')
    );
    assert!(read_identity(duplicate.as_bytes()).is_err());
    assert!(read_identity(&vec![b' '; 4097]).is_err());
    assert!(read_identity(b"{}").is_err());
    assert!(verify_record(&encode(&b, &a).unwrap(), &bytes).is_err());
    let html = render(&raw, Some((&a, &b))).unwrap();
    verify(&raw, html.as_bytes(), Some((&a, &b))).unwrap();
    for bad in [
        format!("{html}\n"),
        html.replace("style", "STYLE"),
        html.replace("Content-Security-Policy", "Removed-Security-Policy"),
        html.replacen('<', "[", 1),
    ] {
        assert!(verify(&raw, bad.as_bytes(), Some((&a, &b))).is_err());
    }
    assert!(verify(&raw, &vec![b'x'; 65537], None).is_err());
    assert!(verify(&raw, b"\xff", None).is_err());
    assert!(verify(&raw, html.as_bytes(), Some((&b, &a))).is_err());
}
#[test]
fn identity_files_compose_with_cli_sources_without_writes() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = pair();
    let raw = encode(&a, &b).unwrap();
    let id = serde_json::to_vec(&identity(&raw).unwrap()).unwrap();
    let html = render(&raw, Some((&a, &b))).unwrap().into_bytes();
    for (n, v) in [
        ("a", a.clone()),
        ("b", b.clone()),
        ("diff", raw.clone()),
        ("id", id.clone()),
        ("html", html.clone()),
    ] {
        std::fs::write(d.path().join(n), v).unwrap();
    }
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
            .current_dir(d.path())
            .args(args)
            .output()
            .unwrap()
    };
    let schema_out = run(&["configuration-diff-identity-schema"]);
    assert!(schema_out.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&schema_out.stdout).unwrap(),
        schema()
    );
    assert!(
        run(&["configuration-diff-check", "diff", "--identity", "id"])
            .status
            .success()
    );
    assert!(
        !run(&[
            "configuration-diff-check",
            "diff",
            "--identity",
            "id",
            "--expected-sha256",
            &"0".repeat(64)
        ])
        .status
        .success()
    );
    for cmd in ["configuration-diff-html", "configuration-diff-csv"] {
        assert!(
            !run(&[
                cmd,
                "diff",
                "--identity",
                "id",
                "--before",
                "b",
                "--after",
                "a",
                "--out",
                "blocked"
            ])
            .status
            .success()
        );
        assert!(!d.path().join("blocked").exists());
        assert!(
            run(&[
                cmd,
                "diff",
                "--identity",
                "id",
                "--before",
                "a",
                "--after",
                "b",
                "--out",
                cmd
            ])
            .status
            .success()
        );
    }
    assert!(
        run(&[
            "configuration-diff-verify",
            "diff",
            "a",
            "b",
            "--identity",
            "id"
        ])
        .status
        .success()
    );
    assert!(
        run(&[
            "configuration-diff-csv-verify",
            "diff",
            "configuration-diff-csv",
            "--identity",
            "id"
        ])
        .status
        .success()
    );
    let args = [
        "configuration-diff-html-verify",
        "diff",
        "html",
        "--identity",
        "id",
        "--before",
        "a",
        "--after",
        "b",
    ];
    assert!(run(&args).status.success());
    assert!(
        !run(&[
            "configuration-diff-html-verify",
            "diff",
            "html",
            "--before",
            "a"
        ])
        .status
        .success()
    );
    std::fs::write(d.path().join("html"), b"changed").unwrap();
    assert!(!run(&args).status.success());
    std::fs::write(d.path().join("html"), &html).unwrap();
    let replaced = encode(&a, &a).unwrap();
    std::fs::write(d.path().join("diff"), &replaced).unwrap();
    std::fs::write(
        d.path().join("id"),
        serde_json::to_vec(&identity(&replaced).unwrap()).unwrap(),
    )
    .unwrap();
    std::fs::write(d.path().join("html"), render(&replaced, None).unwrap()).unwrap();
    assert!(!run(&args).status.success());
    for (n, v) in [
        ("diff", raw.clone()),
        ("id", id.clone()),
        ("html", html.clone()),
    ] {
        std::fs::write(d.path().join(n), v).unwrap();
    }
    assert!(run(&args).status.success());
    for (n, v) in [
        ("a", a),
        ("b", b),
        ("diff", raw),
        ("id", id),
        ("html", html),
    ] {
        assert_eq!(std::fs::read(d.path().join(n)).unwrap(), v);
    }
}
