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
