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

fn catalog_args() -> Vec<&'static str> {
    vec![
        "mcp-catalog",
        "--catalog",
        "catalog.json",
        "--trust",
        "trust.json",
        "--policy",
        "policy.json",
        "--audit",
        "audit.jsonl",
    ]
}

#[test]
fn catalog_pages_dispatch_and_audit_signed_identity() {
    let dir = setup();
    let names: Vec<String> = (0..9).map(|i| format!("tool-{i}")).collect();
    let policy = Policy {
        allowed_tools: names.clone(),
        ..Policy::default()
    };
    std::fs::write(
        dir.path().join("policy.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    let mut entries = Vec::new();
    for name in names.iter().rev() {
        let manifest = format!("{name}.json");
        assert!(
            call(
                dir.path(),
                &[
                    "pack",
                    "--component",
                    "guest.wasm",
                    "--name",
                    name,
                    "--key",
                    "signing.key",
                    "--out",
                    &manifest
                ],
                b""
            )
            .status
            .success()
        );
        entries.push(json!({"manifest":manifest,"component":"guest.wasm"}));
    }
    std::fs::write(
        dir.path().join("catalog.json"),
        json!({"schema_version":1,"packages":entries}).to_string(),
    )
    .unwrap();
    let frames = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/list","params":{"cursor":"tools:8"}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/list","params":{"cursor":"tools:16"}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"tool-0","arguments":{"input":"hello"}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"tool-8","arguments":{"input":"again"}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"unknown","arguments":{"input":"never"}}}),
        json!({"jsonrpc":"2.0","method":"tools/call","params":{"name":"tool-0","arguments":{"input":"never"}}}),
    ];
    let input = frames.iter().map(|f| format!("{f}\n")).collect::<String>();
    let output = call(dir.path(), &catalog_args(), input.as_bytes());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replies: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(replies.len(), 7);
    let first = replies[1]["result"]["tools"].as_array().unwrap();
    assert_eq!(first.len(), 8);
    assert_eq!(first[0]["name"], "tool-0");
    assert_eq!(first[7]["name"], "tool-7");
    assert_eq!(replies[1]["result"]["nextCursor"], "tools:8");
    assert_eq!(replies[2]["result"]["tools"][0]["name"], "tool-8");
    assert!(replies[2]["result"].get("nextCursor").is_none());
    assert_eq!(replies[3]["error"]["code"], -32602);
    assert_eq!(replies[4]["result"]["content"][0]["text"], "HELLO");
    assert_eq!(replies[5]["result"]["content"][0]["text"], "AGAIN");
    assert_eq!(replies[6]["error"]["code"], -32602);
    let audit = std::fs::read(dir.path().join("audit.jsonl")).unwrap();
    assert_eq!(verify_log(&audit).unwrap().0, 2);
    let records: Vec<Value> = std::str::from_utf8(&audit)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(records[0]["receipt"]["tool"], "tool-0");
    assert_eq!(records[1]["receipt"]["tool"], "tool-8");
    assert_ne!(
        records[0]["receipt"]["manifest_sha256"],
        records[1]["receipt"]["manifest_sha256"]
    );
}

#[test]
fn catalog_fails_closed_before_stdio_for_invalid_intake() {
    let dir = setup();
    let entry = json!({"manifest":"signed.json","component":"guest.wasm"});
    for catalog in [
        json!({"schema_version":1,"packages":[]}),
        json!({"schema_version":2,"packages":[entry.clone()]}),
        json!({"schema_version":true,"packages":[entry.clone()]}),
        json!({"schema_version":1,"packages":[entry.clone()],"discover":true}),
        json!({"schema_version":1,"packages":[entry.clone(),entry.clone()]}),
        json!({"schema_version":1,"packages":vec![entry.clone();17]}),
        json!({"schema_version":1,"packages":[{"manifest":"signed.json","component":"guest.wasm","extra":1}]}),
        json!({"schema_version":1,"packages":[entry.clone(),{"manifest":"missing.json","component":"guest.wasm"}]}),
    ] {
        std::fs::write(dir.path().join("catalog.json"), catalog.to_string()).unwrap();
        let output = call(dir.path(), &catalog_args(), b"");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!dir.path().join("audit.jsonl").exists());
    }
    std::fs::write(
        dir.path().join("catalog.json"),
        json!({"schema_version":1,"packages":[entry]}).to_string(),
    )
    .unwrap();
    // Current trust is checked on every startup.
    std::fs::write(dir.path().join("trust.json"), "{}").unwrap();
    assert_eq!(
        call(dir.path(), &catalog_args(), b"").status.code(),
        Some(2)
    );
}
