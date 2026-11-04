use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};
use tempfile::TempDir;
use toolcapsule::{
    Policy,
    storage::{read_frame, verify_log},
};

fn call(dir: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_toolcapsule"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

fn setup() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy("target/fixtures/guest.wasm", dir.path().join("guest.wasm")).unwrap();
    let policy = Policy {
        allowed_tools: vec!["uppercase".into()],
        ..Policy::default()
    };
    std::fs::write(
        dir.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    assert!(
        call(
            dir.path(),
            &["keygen", "--key", "signing.key", "--trust", "trust.json"],
            b""
        )
        .status
        .success()
    );
    assert!(
        call(
            dir.path(),
            &[
                "pack",
                "--component",
                "guest.wasm",
                "--name",
                "uppercase",
                "--key",
                "signing.key",
                "--out",
                "signed.json"
            ],
            b""
        )
        .status
        .success()
    );
    dir
}

fn package(command: &str) -> Vec<&str> {
    vec![
        command,
        "--component",
        "guest.wasm",
        "--manifest",
        "signed.json",
        "--trust",
        "trust.json",
        "--policy",
        "policy.json",
    ]
}

#[test]
fn trust_and_budget_checks_are_read_only_and_distinguish_nonfit_from_invalid() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("trust.json"), b"{}").unwrap();
    let result = call(dir.path(), &["trust-check", "--trust", "trust.json"], b"");
    assert!(result.status.success());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["key_count"], 0);
    assert_eq!(value["denies_all_signers"], true);
    std::fs::write(
        dir.path().join("trust.json"),
        b"{\"private_marker\":\"bad\"}",
    )
    .unwrap();
    let result = call(dir.path(), &["trust-check", "--trust", "trust.json"], b"");
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("private_marker"));
    assert!(
        call(
            dir.path(),
            &["limits-template", "--out", "limits.json"],
            b""
        )
        .status
        .success()
    );
    assert!(
        call(
            dir.path(),
            &["policy-template", "--out", "policy.json"],
            b""
        )
        .status
        .success()
    );
    let args = [
        "budget-fit",
        "--limits",
        "limits.json",
        "--policy",
        "policy.json",
    ];
    let result = call(dir.path(), &args, b"");
    assert!(result.status.success());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["fits"], true);
    assert_eq!(value["package_admission"], "not_checked");
    let mut limits = toolcapsule::Limits::default();
    limits.fuel += 1;
    let bytes = serde_json::to_vec(&limits).unwrap();
    std::fs::write(dir.path().join("limits.json"), &bytes).unwrap();
    let result = call(dir.path(), &args, b"");
    assert_eq!(result.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["exceeded_fields"], serde_json::json!(["fuel"]));
    assert_eq!(
        std::fs::read(dir.path().join("limits.json")).unwrap(),
        bytes
    );
    std::fs::write(dir.path().join("limits.json"), b"{}").unwrap();
    let result = call(dir.path(), &args, b"");
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}

#[test]
fn configuration_checks_are_bounded_read_only_and_do_not_echo_input() {
    let dir = tempfile::tempdir().unwrap();
    for (template, check, option, filename) in [
        ("limits-template", "limits-check", "--limits", "limits.json"),
        ("policy-template", "policy-check", "--policy", "policy.json"),
    ] {
        assert!(
            call(dir.path(), &[template, "--out", filename], b"")
                .status
                .success()
        );
        let before = std::fs::read(dir.path().join(filename)).unwrap();
        let output = call(dir.path(), &[check, option, filename], b"");
        assert!(output.status.success());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["valid"], true);
        assert_eq!(value["schema_version"], 1);
        if check == "policy-check" {
            assert_eq!(value["denies_all_tools"], true);
            assert_eq!(value["allowed_tool_count"], 0);
        }
        assert_eq!(std::fs::read(dir.path().join(filename)).unwrap(), before);
        for invalid in [b"{\"private_marker\":1}".to_vec(), vec![b' '; 65537]] {
            std::fs::write(dir.path().join(filename), invalid).unwrap();
            let output = call(dir.path(), &[check, option, filename], b"");
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(!String::from_utf8_lossy(&output.stderr).contains("private_marker"));
        }
    }
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn templates_roundtrip_and_policy_requires_explicit_authority() {
    let dir = setup();
    let output = call(dir.path(), &["limits-template"], b"");
    assert!(output.status.success());
    let limits: toolcapsule::Limits = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(limits, toolcapsule::Limits::default());
    let emit = ["limits-template", "--out", "limits.json"];
    assert!(call(dir.path(), &emit, b"").status.success());
    let original = std::fs::read(dir.path().join("limits.json")).unwrap();
    assert_eq!(call(dir.path(), &emit, b"").status.code(), Some(2));
    assert_eq!(
        std::fs::read(dir.path().join("limits.json")).unwrap(),
        original
    );
    let emit = [
        "policy-template",
        "--limits",
        "limits.json",
        "--out",
        "host.json",
    ];
    assert!(call(dir.path(), &emit, b"").status.success());
    assert_eq!(call(dir.path(), &emit, b"").status.code(), Some(2));
    let mut policy: Policy =
        serde_json::from_slice(&std::fs::read(dir.path().join("host.json")).unwrap()).unwrap();
    assert!(policy.allowed_tools.is_empty());
    assert_eq!(policy.max_limits, limits);
    std::fs::copy(dir.path().join("host.json"), dir.path().join("policy.json")).unwrap();
    assert_eq!(
        call(dir.path(), &package("verify"), b"").status.code(),
        Some(2)
    );
    policy.allowed_tools.push("uppercase".into());
    std::fs::write(
        dir.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    assert!(call(dir.path(), &package("verify"), b"").status.success());
}

#[test]
fn policy_template_rejects_bad_input_before_output() {
    let dir = tempfile::tempdir().unwrap();
    let output = call(dir.path(), &["policy-template"], b"");
    let policy: Policy = serde_json::from_slice(&output.stdout).unwrap();
    assert!(policy.allowed_tools.is_empty());
    for bytes in [
        b"{}".to_vec(),
        vec![b' '; 4097],
        serde_json::to_vec(&toolcapsule::Limits {
            fuel: 0,
            ..Default::default()
        })
        .unwrap(),
    ] {
        std::fs::write(dir.path().join("limits.json"), bytes).unwrap();
        let output = call(
            dir.path(),
            &[
                "policy-template",
                "--limits",
                "limits.json",
                "--out",
                "host.json",
            ],
            b"",
        );
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!dir.path().join("host.json").exists());
    }
}

#[test]
fn pack_explicit_limits_are_signed_enforced_and_cannot_grant_host_authority() {
    let dir = setup();
    let limits = toolcapsule::Limits {
        input_bytes: 3,
        output_bytes: 100,
        ..Default::default()
    };
    std::fs::write(
        dir.path().join("limits.json"),
        serde_json::to_vec(&limits).unwrap(),
    )
    .unwrap();
    let pack = [
        "pack",
        "--component",
        "guest.wasm",
        "--name",
        "uppercase",
        "--key",
        "signing.key",
        "--limits",
        "limits.json",
        "--out",
        "custom.json",
    ];
    assert!(call(dir.path(), &pack, b"").status.success());
    std::fs::copy(
        dir.path().join("custom.json"),
        dir.path().join("signed.json"),
    )
    .unwrap();
    let output = call(dir.path(), &package("preflight"), b"");
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["manifest"]["limits"],
        serde_json::to_value(&limits).unwrap()
    );
    let mut run = package("run");
    run.extend(["--audit", "audit.jsonl"]);
    assert!(call(dir.path(), &run, b"abc").status.success());
    assert_eq!(call(dir.path(), &run, b"abcd").status.code(), Some(2));
    let mut policy: Policy =
        serde_json::from_slice(&std::fs::read(dir.path().join("policy.json")).unwrap()).unwrap();
    policy.max_limits.input_bytes = 2;
    std::fs::write(
        dir.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    assert_eq!(
        call(dir.path(), &package("verify"), b"").status.code(),
        Some(2)
    );
    assert_eq!(call(dir.path(), &pack, b"").status.code(), Some(2));
}

#[test]
fn pack_rejects_invalid_limits_before_creating_output() {
    let dir = setup();
    let valid = serde_json::to_value(toolcapsule::Limits::default()).unwrap();
    let mut invalid = Vec::new();
    for (field, value) in [
        ("fuel", 0),
        ("memory_bytes", 65535),
        ("timeout_ms", 9),
        ("input_bytes", 0),
        ("output_bytes", 0),
        ("extra", 1),
    ] {
        let mut data = valid.clone();
        data[field] = json!(value);
        invalid.push(serde_json::to_vec(&data).unwrap());
    }
    invalid.extend([b"{}".to_vec(), vec![b' '; 4097]]);
    for bytes in invalid {
        std::fs::write(dir.path().join("limits.json"), bytes).unwrap();
        let output = call(
            dir.path(),
            &[
                "pack",
                "--component",
                "guest.wasm",
                "--name",
                "uppercase",
                "--key",
                "signing.key",
                "--limits",
                "limits.json",
                "--out",
                "rejected.json",
            ],
            b"",
        );
        assert_eq!(output.status.code(), Some(2));
        assert!(!dir.path().join("rejected.json").exists());
    }
}

#[test]
fn cli_preflight_checks_interfaces_after_admission_without_outputs() {
    let dir = setup();
    let output = call(dir.path(), &package("preflight"), b"");
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["compiled"], true);
    assert_eq!(result["executed"], false);
    assert_eq!(result["manifest"]["name"], "uppercase");
    assert!(!dir.path().join("audit.jsonl").exists());
    std::fs::write(dir.path().join("guest.wasm"), b"invalid component").unwrap();
    let denied = call(dir.path(), &package("preflight"), b"");
    assert_eq!(denied.status.code(), Some(2));
    assert!(denied.stdout.is_empty());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("digest mismatch"));
    assert!(
        call(
            dir.path(),
            &[
                "pack",
                "--component",
                "guest.wasm",
                "--name",
                "uppercase",
                "--key",
                "signing.key",
                "--out",
                "invalid.json"
            ],
            b""
        )
        .status
        .success()
    );
    std::fs::copy(
        dir.path().join("invalid.json"),
        dir.path().join("signed.json"),
    )
    .unwrap();
    assert!(call(dir.path(), &package("verify"), b"").status.success());
    let incompatible = call(dir.path(), &package("preflight"), b"");
    assert_eq!(incompatible.status.code(), Some(2));
    assert!(incompatible.stdout.is_empty());
    assert!(String::from_utf8_lossy(&incompatible.stderr).contains("interface validation failed"));
    std::fs::write(dir.path().join("trust.json"), b"{}").unwrap();
    let revoked = call(dir.path(), &package("preflight"), b"");
    assert!(String::from_utf8_lossy(&revoked.stderr).contains("untrusted signer"));
}

#[test]
fn cli_sign_verify_run_replay_and_audit_round_trip() {
    let dir = setup();
    assert!(call(dir.path(), &package("verify"), b"").status.success());
    let mut args = package("run");
    args.extend([
        "--audit",
        "audit.jsonl",
        "--receipt",
        "receipt.json",
        "--seed",
        "42",
    ]);
    let output = call(dir.path(), &args, b"hello\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["stdout"], "HELLO\n");
    let mut replay = package("replay");
    replay.extend(["--receipt", "receipt.json"]);
    assert!(call(dir.path(), &replay, b"hello\n").status.success());
    assert!(!call(dir.path(), &replay, b"changed\n").status.success());
    let audit = call(dir.path(), &["audit-verify", "--audit", "audit.jsonl"], b"");
    assert!(audit.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&audit.stdout).unwrap()["entries"],
        1
    );
    let log = std::fs::read(dir.path().join("audit.jsonl")).unwrap();
    assert!(!String::from_utf8_lossy(&log).contains("hello"));
}

#[test]
fn cli_never_overwrites_keys_and_rejects_changed_components() {
    let dir = setup();
    let key = std::fs::read(dir.path().join("signing.key")).unwrap();
    assert!(
        !call(
            dir.path(),
            &["keygen", "--key", "signing.key", "--trust", "trust.json"],
            b""
        )
        .status
        .success()
    );
    assert_eq!(std::fs::read(dir.path().join("signing.key")).unwrap(), key);
    std::fs::write(dir.path().join("guest.wasm"), b"tampered").unwrap();
    let output = call(dir.path(), &package("verify"), b"");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("digest mismatch"));
    assert!(output.stdout.is_empty());
}

#[test]
fn mcp_lifecycle_calls_errors_and_notifications_over_real_stdio() {
    let dir = setup();
    let messages = vec![
        json!({"jsonrpc":"2.0","id":0,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"future-version","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"uppercase","arguments":{"input":"hello","seed":42}}}),
        json!({"jsonrpc":"2.0","method":"tools/call","params":{"name":"uppercase","arguments":{"input":"notification must not execute"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"uppercase","arguments":{"input":3}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"missing","arguments":{"input":"x"}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"uppercase","arguments":{"input":"fail"}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"unknown"}),
        json!({"jsonrpc":"2.0","id":8,"method":"ping"}),
        json!([]),
    ];
    let mut input = messages
        .iter()
        .map(|v| serde_json::to_string(v).unwrap() + "\n")
        .collect::<String>();
    input.push_str("{broken\n");
    let mut args = package("mcp");
    args.extend(["--audit", "audit.jsonl"]);
    let output = call(dir.path(), &args, input.as_bytes());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let replies: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(replies.len(), 11);
    assert_eq!(replies[0]["error"]["code"], -32000);
    assert_eq!(replies[1]["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(replies[2]["result"]["tools"][0]["name"], "uppercase");
    assert_eq!(replies[3]["result"]["content"][0]["text"], "HELLO");
    assert_eq!(replies[4]["result"]["isError"], true);
    assert_eq!(replies[5]["error"]["code"], -32602);
    assert_eq!(replies[6]["result"]["isError"], true);
    assert_eq!(replies[7]["error"]["code"], -32601);
    assert_eq!(replies[8]["result"], json!({}));
    assert_eq!(replies[9]["error"]["code"], -32600);
    assert_eq!(replies[10]["error"]["code"], -32700);
    let (entries, _) = verify_log(&std::fs::read(dir.path().join("audit.jsonl")).unwrap()).unwrap();
    assert_eq!(entries, 2);
}

#[test]
fn framing_rejects_oversize_and_truncated_messages() {
    assert!(toolcapsule::storage::read_bounded(std::io::empty(), usize::MAX).is_err());
    assert!(read_frame(&mut std::io::Cursor::new(b""), usize::MAX).is_err());
    let mut reader = std::io::Cursor::new(b"123456\n");
    assert!(read_frame(&mut reader, 4).is_err());
    assert!(read_frame(&mut std::io::Cursor::new(b"{}"), 4).is_err());
    assert_eq!(
        read_frame(&mut std::io::Cursor::new(b"{}\n"), 4).unwrap(),
        Some(b"{}\n".to_vec())
    );
    assert!(
        read_frame(&mut std::io::Cursor::new(b""), 4)
            .unwrap()
            .is_none()
    );
}

#[test]
fn truncated_or_tampered_audits_are_rejected() {
    let dir = setup();
    let mut args = package("run");
    args.extend(["--audit", "audit.jsonl"]);
    assert!(call(dir.path(), &args, b"hello").status.success());
    assert!(call(dir.path(), &args, b"again").status.success());
    let bytes = std::fs::read(dir.path().join("audit.jsonl")).unwrap();
    assert_eq!(verify_log(&bytes).unwrap().0, 2);
    assert!(verify_log(&bytes[..bytes.len() - 1]).is_err());
    let text = String::from_utf8(bytes)
        .unwrap()
        .replace("uppercase", "different");
    std::fs::write(dir.path().join("audit.jsonl"), text).unwrap();
    assert!(!call(dir.path(), &args, b"hello").status.success());
}

#[test]
fn mcp_errors_omit_unknown_ids_and_preserve_known_ids() {
    let dir = setup();
    let mut input = String::from("{broken\n[]\n");
    for id in [Value::Null, json!(true), json!({}), json!([]), json!(1.5)] {
        input.push_str(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": id, "method": "ping"
            }))
            .unwrap(),
        );
        input.push('\n');
    }
    for id in [json!(0), json!("request-1")] {
        input.push_str(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": id, "method": "ping", "params": []
            }))
            .unwrap(),
        );
        input.push('\n');
        input.push_str(&serde_json::to_string(&json!({"jsonrpc": "2.0", "id": id})).unwrap());
        input.push('\n');
    }
    input.push_str("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n");
    let mut args = package("mcp");
    args.extend(["--audit", "audit.jsonl"]);
    let output = call(dir.path(), &args, input.as_bytes());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replies: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(replies.len(), 11);
    assert_eq!(replies[0]["error"]["code"], -32700);
    for reply in &replies[..7] {
        assert!(
            reply.get("id").is_none(),
            "unknown request ID must be omitted: {reply}"
        );
        assert!(reply.get("error").is_some());
        assert!(reply.get("result").is_none());
    }
    for (reply, id, code) in [
        (&replies[7], json!(0), -32602),
        (&replies[8], json!(0), -32600),
        (&replies[9], json!("request-1"), -32602),
        (&replies[10], json!("request-1"), -32600),
    ] {
        assert_eq!(reply["id"], id);
        assert_eq!(reply["error"]["code"], code);
    }
    assert!(!dir.path().join("audit.jsonl").exists());
}

#[test]
fn cli_checkpoint_export_and_trusted_verification() {
    let dir = setup();
    let mut run = package("run");
    run.extend(["--audit", "audit.jsonl"]);
    assert!(call(dir.path(), &run, b"first").status.success());
    let export = [
        "audit-checkpoint",
        "--audit",
        "audit.jsonl",
        "--out",
        "anchor.json",
    ];
    let output = call(dir.path(), &export, b"");
    assert!(output.status.success());
    let anchor = std::fs::read(dir.path().join("anchor.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        serde_json::from_slice::<Value>(&anchor).unwrap()
    );
    assert_eq!(call(dir.path(), &export, b"").status.code(), Some(2));
    assert_eq!(
        anchor,
        std::fs::read(dir.path().join("anchor.json")).unwrap()
    );
    let verify = [
        "audit-verify",
        "--audit",
        "audit.jsonl",
        "--checkpoint",
        "anchor.json",
    ];
    let mut exact = verify.to_vec();
    exact.push("--exact");
    assert!(call(dir.path(), &exact, b"").status.success());
    assert!(call(dir.path(), &run, b"second").status.success());
    assert!(call(dir.path(), &verify, b"").status.success());
    assert_eq!(call(dir.path(), &exact, b"").status.code(), Some(2));
    assert_eq!(
        call(
            dir.path(),
            &["audit-verify", "--audit", "audit.jsonl", "--exact"],
            b""
        )
        .status
        .code(),
        Some(2)
    );
    std::fs::write(dir.path().join("anchor.json"), vec![b' '; 4097]).unwrap();
    assert_eq!(call(dir.path(), &verify, b"").status.code(), Some(2));
    std::fs::write(dir.path().join("audit.jsonl"), b"broken\n").unwrap();
    let output = call(
        dir.path(),
        &[
            "audit-checkpoint",
            "--audit",
            "audit.jsonl",
            "--out",
            "bad.json",
        ],
        b"",
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!dir.path().join("bad.json").exists());
}

#[test]
fn cli_signed_checkpoint_authentication_and_prefix_modes() {
    let dir = setup();
    let mut run = package("run");
    run.extend(["--audit", "audit.jsonl"]);
    assert!(call(dir.path(), &run, b"first").status.success());
    assert!(
        call(
            dir.path(),
            &[
                "audit-checkpoint",
                "--audit",
                "audit.jsonl",
                "--out",
                "anchor.json"
            ],
            b""
        )
        .status
        .success()
    );
    let sign = [
        "audit-checkpoint-sign",
        "--checkpoint",
        "anchor.json",
        "--key",
        "signing.key",
        "--out",
        "signed-anchor.json",
    ];
    assert!(call(dir.path(), &sign, b"").status.success());
    let original = std::fs::read(dir.path().join("signed-anchor.json")).unwrap();
    assert_eq!(call(dir.path(), &sign, b"").status.code(), Some(2));
    assert_eq!(
        original,
        std::fs::read(dir.path().join("signed-anchor.json")).unwrap()
    );
    let verify = [
        "audit-verify",
        "--audit",
        "audit.jsonl",
        "--signed-checkpoint",
        "signed-anchor.json",
        "--trust",
        "trust.json",
    ];
    let mut exact = verify.to_vec();
    exact.push("--exact");
    assert!(call(dir.path(), &exact, b"").status.success());
    assert!(call(dir.path(), &run, b"second").status.success());
    assert!(call(dir.path(), &verify, b"").status.success());
    assert_eq!(call(dir.path(), &exact, b"").status.code(), Some(2));
    assert_eq!(call(dir.path(), &verify[..5], b"").status.code(), Some(2));
    let mut conflicting = verify.to_vec();
    conflicting.extend(["--checkpoint", "anchor.json"]);
    assert_eq!(call(dir.path(), &conflicting, b"").status.code(), Some(2));
    std::fs::write(dir.path().join("trust.json"), b"{}").unwrap();
    let revoked = call(dir.path(), &verify, b"");
    assert_eq!(revoked.status.code(), Some(2));
    assert!(revoked.stdout.is_empty());
}
