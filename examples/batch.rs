//! Usage: cargo run --example batch -- signed.json tool.wasm trust.json policy.json
use std::{io, path::Path, sync::Arc};
use toolcapsule::{
    Policy, TrustStore, VerifiedPackage,
    audit::Receipt,
    batch::{Invocation, MAX_INPUT_BYTES, run_batch},
    package::MAX_ENVELOPE_BYTES,
    runtime::Runtime,
    storage::{read_bounded, read_file, read_json},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("expected: manifest component trust policy".into());
    }
    let trust: TrustStore = read_json(Path::new(&args[2]), 65536)?;
    let policy: Policy = read_json(Path::new(&args[3]), 65536)?;
    let package = VerifiedPackage::verify(
        &read_file(Path::new(&args[0]), MAX_ENVELOPE_BYTES)?,
        read_file(Path::new(&args[1]), policy.max_component_bytes)?,
        &trust,
        &policy,
    )?;
    let tool = Arc::new(Runtime::new()?.prepare(package)?);
    let input = read_bounded(
        io::stdin().lock(),
        tool.manifest().limits.input_bytes.min(MAX_INPUT_BYTES / 8),
    )?;
    let invocations = (0..8)
        .map(|seed| Invocation {
            input: input.clone(),
            seed,
        })
        .collect();
    let results = run_batch(tool.clone(), invocations, 4).await?;
    let receipts: Vec<_> = results
        .iter()
        .enumerate()
        .map(|(seed, result)| Receipt::new(&tool, &input, seed as u64, result))
        .collect();
    println!("{}", serde_json::to_string(&receipts)?);
    Ok(())
}
