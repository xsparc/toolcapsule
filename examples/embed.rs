//! Usage: cargo run --example embed -- signed.json tool.wasm trust.json policy.json
use std::{io, path::Path};
use toolcapsule::{
    Policy, TrustStore, VerifiedPackage,
    audit::Receipt,
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
    let tool = Runtime::new()?.prepare(package)?;
    let input = read_bounded(io::stdin().lock(), tool.manifest().limits.input_bytes)?;
    let result = tool.run(&input, 42).await?;
    let receipt = Receipt::new(&tool, &input, 42, &result);
    // A SaaS worker would send this receipt to its tenant's durable audit sink.
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}
