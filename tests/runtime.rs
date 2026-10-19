use ed25519_dalek::SigningKey;
use std::sync::Arc;
use toolcapsule::batch::{Invocation, run_batch};
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
async fn pool_shares_capacity_and_releases_only_after_batch_cleanup() {
    use toolcapsule::pool::ExecutionPool;
    let tool = Arc::new(tool(
        std::fs::read("target/fixtures/guest.wasm").unwrap(),
        Limits::default(),
    ));
    let pool = ExecutionPool::new(2).unwrap();
    let clone = pool.clone();
    let mut batch = Box::pin(
        pool.run_batch(
            tool.clone(),
            (0..8)
                .map(|seed| Invocation {
                    input: b"sleep".to_vec(),
                    seed,
                })
                .collect(),
            2,
        ),
    );
    tokio::select! {
        _ = &mut batch => panic!("sleeping batch unexpectedly completed"),
        _ = tokio::time::sleep(std::time::Duration::from_millis(20)) => {}
    }
    assert!(matches!(
        clone.run(&tool, b"hello", 0).await,
        Err(Error::Busy)
    ));
    assert!(matches!(
        clone
            .run_batch(
                tool.clone(),
                vec![Invocation {
                    input: vec![],
                    seed: 0
                }],
                1
            )
            .await,
        Err(Error::Busy)
    ));
    drop(batch);
    // Aborted tasks still own their reservation until the executor drops them.
    assert!(matches!(
        clone.run(&tool, b"hello", 0).await,
        Err(Error::Busy)
    ));
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        clone.run(&tool, b"hello", 0).await.unwrap().stdout,
        b"HELLO"
    );
}

#[tokio::test]
async fn pool_cancelled_batches_release_capacity_for_reuse_and_replay() {
    use toolcapsule::pool::ExecutionPool;
    let tool = Arc::new(tool(
        std::fs::read("target/fixtures/guest.wasm").unwrap(),
        Limits::default(),
    ));
    let pool = ExecutionPool::new(2).unwrap();
    assert!(
        tokio::time::timeout(
            std::time::Duration::from_millis(20),
            pool.run_batch(
                tool.clone(),
                (0..8)
                    .map(|seed| Invocation {
                        input: b"sleep".to_vec(),
                        seed
                    })
                    .collect(),
                2
            )
        )
        .await
        .is_err()
    );
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    let result = pool.run(&tool, b"hello\n", 9).await.unwrap();
    assert_eq!(result.stdout, b"HELLO\n");
    Receipt::new(&tool, b"hello\n", 9, &result)
        .replay(&tool, b"hello\n")
        .await
        .unwrap();
    assert_eq!(Arc::strong_count(&tool), 1);
    assert!(
        pool.run_batch(tool.clone(), vec![], 1)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        pool.run_batch(
            tool.clone(),
            vec![Invocation {
                input: vec![],
                seed: 0
            }],
            0
        )
        .await,
        Err(Error::Policy)
    ));
    assert!(matches!(
        pool.run(&tool, &vec![0; 65537], 0).await,
        Err(Error::InputLimit)
    ));
    assert!(matches!(
        pool.run_batch(
            tool.clone(),
            (0..3)
                .map(|seed| Invocation {
                    input: vec![],
                    seed
                })
                .collect(),
            3
        )
        .await,
        Err(Error::Busy)
    ));
    let results = pool
        .run_batch(
            tool,
            (0..3)
                .map(|seed| Invocation {
                    input: b"probe".to_vec(),
                    seed,
                })
                .collect(),
            2,
        )
        .await
        .unwrap();
    assert!(results.iter().all(|r| r.stdout == b"capabilities-denied\n"));
}

#[tokio::test]
async fn batch_real_guest_preserves_order_outcomes_and_replay() {
    let tool = Arc::new(tool(
        std::fs::read("target/fixtures/guest.wasm").unwrap(),
        Limits {
            timeout_ms: 50,
            ..Limits::default()
        },
    ));
    let inputs: Vec<Vec<u8>> = vec![
        b"sleep".to_vec(),
        b"hello\n".to_vec(),
        b"fail".to_vec(),
        b"probe".to_vec(),
    ];
    let jobs = inputs
        .iter()
        .enumerate()
        .map(|(i, input)| Invocation {
            input: input.clone(),
            seed: i as u64,
        })
        .collect();
    let results = run_batch(tool.clone(), jobs, 4).await.unwrap();
    assert_eq!(results[0].outcome, Outcome::TimeLimit);
    assert_eq!(results[1].stdout, b"HELLO\n");
    assert_eq!(results[2].outcome, Outcome::GuestError);
    assert_eq!(results[3].stdout, b"capabilities-denied\n");
    Receipt::new(&tool, &inputs[1], 1, &results[1])
        .replay(&tool, &inputs[1])
        .await
        .unwrap();
    assert_eq!(Arc::strong_count(&tool), 1);
}

#[tokio::test]
async fn batch_admission_rejects_counts_bytes_and_overflow() {
    let tool = Arc::new(tool(command("i32.const 0", ""), Limits::default()));
    assert!(run_batch(tool.clone(), vec![], 1).await.unwrap().is_empty());
    for concurrency in [0, 17] {
        assert!(matches!(
            run_batch(tool.clone(), vec![], concurrency).await,
            Err(Error::Policy)
        ));
    }
    let jobs = (0..65)
        .map(|_| Invocation {
            input: vec![],
            seed: 0,
        })
        .collect();
    assert!(matches!(
        run_batch(tool.clone(), jobs, 1).await,
        Err(Error::Policy)
    ));
    assert!(matches!(
        run_batch(
            tool,
            vec![Invocation {
                input: vec![0; 65537],
                seed: 0
            }],
            1
        )
        .await,
        Err(Error::InputLimit)
    ));
    let large = Arc::new(self::tool(
        command("i32.const 0", ""),
        Limits {
            input_bytes: 5 * 1024 * 1024,
            ..Limits::default()
        },
    ));
    assert!(matches!(
        run_batch(
            large,
            vec![Invocation {
                input: vec![0; 4 * 1024 * 1024 + 1],
                seed: 0
            }],
            1
        )
        .await,
        Err(Error::Policy)
    ));
    let huge = Arc::new(self::tool(
        command("i32.const 0", ""),
        Limits {
            output_bytes: usize::MAX,
            ..Limits::default()
        },
    ));
    assert!(matches!(
        run_batch(
            huge,
            vec![Invocation {
                input: vec![],
                seed: 0
            }],
            1
        )
        .await,
        Err(Error::Policy)
    ));
}

#[tokio::test]
async fn concurrent_batch_instances_have_fresh_globals() {
    let bytes = command(
        "global.get 0 global.get 0 i32.const 1 i32.add global.set 0",
        "(global (mut i32) (i32.const 0))",
    );
    let tool = Arc::new(tool(bytes, Limits::default()));
    let jobs = (0..64)
        .map(|seed| Invocation {
            input: vec![],
            seed,
        })
        .collect();
    let results = run_batch(tool, jobs, 16).await.unwrap();
    assert_eq!(results.len(), 64);
    assert!(results.iter().all(|r| r.outcome == Outcome::Success));
}

#[tokio::test]
async fn batch_cancellation_releases_real_guest_tasks() {
    let tool = Arc::new(tool(
        std::fs::read("target/fixtures/guest.wasm").unwrap(),
        Limits {
            timeout_ms: 5000,
            ..Limits::default()
        },
    ));
    let jobs = (0..8)
        .map(|seed| Invocation {
            input: b"sleep".to_vec(),
            seed,
        })
        .collect();
    assert!(
        tokio::time::timeout(
            std::time::Duration::from_millis(30),
            run_batch(tool.clone(), jobs, 2)
        )
        .await
        .is_err()
    );
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    assert_eq!(Arc::strong_count(&tool), 1);
    assert_eq!(
        tool.run(b"still usable", 1).await.unwrap().stdout,
        b"STILL USABLE"
    );
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
