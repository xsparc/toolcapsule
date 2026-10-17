use ed25519_dalek::SigningKey;
use toolcapsule::{
    audit::{AuditEntry, Receipt},
    package::digest,
    runtime::{Outcome, PreparedTool, Runtime},
    *,
};

fn tool(bytes: Vec<u8>, limits: Limits) -> PreparedTool {
    let key = SigningKey::from_bytes(&[9; 32]);
    let manifest = Manifest {
        schema_version: 1,
        name: "fixture".into(),
        version: "1".into(),
        description: "Test fixture".into(),
        abi: "wasi:cli/command@0.2".into(),
        component_sha256: digest(&bytes),
        capabilities: Capabilities::default(),
        limits: limits.clone(),
    };
    let envelope =
        serde_json::to_vec(&SignedManifest::sign(&manifest, "fixture", &key).unwrap()).unwrap();
    let trust = [(
        "fixture".into(),
        hex::encode(key.verifying_key().to_bytes()),
    )]
    .into();
    let policy = Policy {
        max_limits: limits,
        allowed_tools: vec!["fixture".into()],
        ..Policy::default()
    };
    Runtime::new()
        .unwrap()
        .prepare(VerifiedPackage::verify(&envelope, bytes, &trust, &policy).unwrap())
        .unwrap()
}

fn command(body: &str, memory: &str) -> Vec<u8> {
    wat::parse_str(format!(
        r#"(component
      (core module $m {memory} (func (export "run") (result i32) {body}))
      (core instance $i (instantiate $m))
      (func $run (result (result)) (canon lift (core func $i "run")))
      (instance $cli (export "run" (func $run)))
      (export "wasi:cli/run@0.2.0" (instance $cli)))"#
    ))
    .unwrap()
}

#[tokio::test]
async fn fuel_exhaustion_and_async_deadline_stop_infinite_loops() {
    let bytes = command("(loop $l br $l) i32.const 0", "");
    let low = tool(
        bytes.clone(),
        Limits {
            fuel: 1000,
            ..Limits::default()
        },
    );
    assert_eq!(
        low.run(b"", 1).await.unwrap().outcome,
        Outcome::FuelExhausted
    );
    let high = tool(
        bytes,
        Limits {
            fuel: u64::MAX,
            timeout_ms: 20,
            ..Limits::default()
        },
    );
    assert_eq!(high.run(b"", 1).await.unwrap().outcome, Outcome::TimeLimit);
}

#[tokio::test]
async fn memory_growth_is_limited_and_input_is_bounded() {
    let tool = tool(
        command("i32.const 2 memory.grow drop i32.const 0", "(memory 1)"),
        Limits {
            memory_bytes: 65536,
            ..Limits::default()
        },
    );
    assert_eq!(tool.run(b"", 0).await.unwrap().outcome, Outcome::Trap);
    assert!(matches!(
        tool.run(&vec![0; 65537], 0).await,
        Err(Error::InputLimit)
    ));
}

#[tokio::test]
async fn audit_chain_and_replay_detect_tampering() {
    let tool = tool(command("i32.const 0", ""), Limits::default());
    let result = tool.run(b"sensitive input", 4).await.unwrap();
    assert_eq!(result.outcome, Outcome::Success);
    let receipt = Receipt::new(&tool, b"sensitive input", 4, &result);
    assert!(receipt.replay(&tool, b"sensitive input").await.is_ok());
    assert!(receipt.replay(&tool, b"changed").await.is_err());
    let mut entry = AuditEntry::new(0, String::new(), receipt).unwrap();
    entry.verify(0, "").unwrap();
    assert!(
        !serde_json::to_string(&entry)
            .unwrap()
            .contains("sensitive input")
    );
    entry.receipt.fuel_consumed += 1;
    assert!(entry.verify(0, "").is_err());
    assert!(
        entry
            .receipt
            .replay(&tool, b"sensitive input")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn fresh_instances_do_not_share_mutable_globals() {
    let tool = tool(
        command(
            "global.get 0 i32.const 1 i32.add global.set 0 global.get 0 i32.const 1 i32.sub",
            "(global (mut i32) (i32.const 0))",
        ),
        Limits::default(),
    );
    for _ in 0..3 {
        assert_eq!(tool.run(b"", 0).await.unwrap().outcome, Outcome::Success);
    }
}

// Required integration fixture: build instructions are in docs/development.md.
#[tokio::test]
async fn real_wasi_command_denies_host_access_and_cancels_host_waits() {
    let bytes =
        std::fs::read("target/fixtures/guest.wasm").expect("build the WASI guest fixture first");
    let tool = tool(
        bytes,
        Limits {
            timeout_ms: 100,
            ..Limits::default()
        },
    );
    let result = tool.run(b"hello\n", 42).await.unwrap();
    assert_eq!(result.outcome, Outcome::Success);
    assert_eq!(result.stdout, b"HELLO\n");
    let receipt = Receipt::new(&tool, b"hello\n", 42, &result);
    receipt.replay(&tool, b"hello\n").await.unwrap();
    let probe = tool.run(b"probe", 42).await.unwrap();
    assert_eq!(
        probe.outcome,
        Outcome::Success,
        "{:?}",
        String::from_utf8_lossy(&probe.stderr)
    );
    assert_eq!(probe.stdout, b"capabilities-denied\n");
    assert_eq!(
        tool.run(b"sleep", 42).await.unwrap().outcome,
        Outcome::TimeLimit
    );
    assert_eq!(
        tool.run(b"fail", 42).await.unwrap().outcome,
        Outcome::GuestError
    );
    let flood = tool.run(b"flood", 42).await.unwrap();
    assert!(flood.stdout.len() <= 65536);
    assert!(flood.stderr.len() <= 65536);
}
