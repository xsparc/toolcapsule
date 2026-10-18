//! Synthetic uppercase fixture benchmark; JSON stdout contains no host identity or paths.
use clap::Parser;
use ed25519_dalek::SigningKey;
use serde::Serialize;
use std::{path::PathBuf, time::Instant};
use toolcapsule::{
    Capabilities, Limits, Manifest, Policy, SignedManifest, TrustStore, VerifiedPackage,
    package::digest,
    runtime::{Outcome, PROFILE, RunResult, Runtime},
    storage::read_file,
};

#[derive(Parser)]
struct Args {
    /// Locally compiled examples/guest.rs WASI component.
    component: PathBuf,
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=100))]
    samples: u32,
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(0..=100))]
    warmup: u32,
}

#[derive(Debug, Serialize)]
struct Timing {
    samples: usize,
    min_ns: u128,
    median_ns: u128,
    p95_ns: u128,
    max_ns: u128,
    total_ns: u128,
}

fn summarize(mut values: Vec<u128>) -> Timing {
    assert!(!values.is_empty());
    values.sort_unstable();
    let n = values.len();
    Timing {
        samples: n,
        min_ns: values[0],
        // Nearest-rank percentiles, including the median, without interpolation.
        median_ns: values[n.div_ceil(2) - 1],
        p95_ns: values[(95 * n).div_ceil(100) - 1],
        max_ns: values[n - 1],
        total_ns: values.iter().sum(),
    }
}

fn check(result: &RunResult, expected: Option<&RunResult>) -> Result<(), &'static str> {
    if result.outcome != Outcome::Success
        || result.stdout != b"BENCHMARK\n"
        || !result.stderr.is_empty()
        || expected.is_some_and(|previous| previous != result)
    {
        return Err("fixture failed or execution changed; no timing report published");
    }
    Ok(())
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    profile: &'static str,
    build_mode: &'static str,
    component_sha256: String,
    component_bytes: usize,
    input_bytes: usize,
    seed: u64,
    warmup: u32,
    engine_setup_ns: u128,
    verification_with_copy: Timing,
    preparation: Timing,
    first_invocation: Timing,
    reused_invocation: Timing,
    fuel_consumed: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let bytes = read_file(&args.component, 16 * 1024 * 1024)?;
    let limits = Limits {
        fuel: 10_000_000,
        timeout_ms: 5000,
        ..Limits::default()
    };
    let manifest = Manifest {
        schema_version: 1,
        name: "benchmark".into(),
        version: "1".into(),
        description: "Synthetic uppercase benchmark".into(),
        abi: "wasi:cli/command@0.2".into(),
        component_sha256: digest(&bytes),
        capabilities: Capabilities::default(),
        limits: limits.clone(),
    };
    // Public fixture key only. This example never reads an operator trust store.
    let key = SigningKey::from_bytes(&[9; 32]);
    let envelope = serde_json::to_vec(&SignedManifest::sign(&manifest, "fixture", &key)?)?;
    let trust: TrustStore = [(
        "fixture".into(),
        hex::encode(key.verifying_key().to_bytes()),
    )]
    .into();
    let policy = Policy {
        max_limits: limits,
        allowed_tools: vec!["benchmark".into()],
        ..Policy::default()
    };
    let input = b"benchmark\n";
    let start = Instant::now();
    let runtime = Runtime::new()?;
    let engine_setup_ns = start.elapsed().as_nanos();
    let (mut verification, mut preparation, mut first, mut reused) =
        (vec![], vec![], vec![], vec![]);
    let mut expected = None;
    let mut retained = None;
    for _ in 0..args.samples {
        let start = Instant::now();
        let package = VerifiedPackage::verify(&envelope, bytes.clone(), &trust, &policy)?;
        verification.push(start.elapsed().as_nanos());
        let start = Instant::now();
        let tool = runtime.prepare(package)?;
        preparation.push(start.elapsed().as_nanos());
        let start = Instant::now();
        let result = tool.run(input, 42).await?;
        first.push(start.elapsed().as_nanos());
        check(&result, expected.as_ref())?;
        expected = Some(result);
        retained = Some(tool);
    }
    let tool = retained.expect("positive samples");
    for i in 0..args.warmup + args.samples {
        let start = Instant::now();
        let result = tool.run(input, 42).await?;
        let elapsed = start.elapsed().as_nanos();
        check(&result, expected.as_ref())?;
        if i >= args.warmup {
            reused.push(elapsed);
        }
    }
    let report = Report {
        schema_version: 1,
        profile: PROFILE,
        build_mode: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        component_sha256: manifest.component_sha256,
        component_bytes: bytes.len(),
        input_bytes: input.len(),
        seed: 42,
        warmup: args.warmup,
        engine_setup_ns,
        verification_with_copy: summarize(verification),
        preparation: summarize(preparation),
        first_invocation: summarize(first),
        reused_invocation: summarize(reused),
        fuel_consumed: expected.expect("positive samples").fuel_consumed,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_rank_handles_singleton_and_unsorted_samples() {
        assert_eq!(summarize(vec![7]).p95_ns, 7);
        let stats = summarize((1..=20).rev().collect());
        assert_eq!(
            (
                stats.min_ns,
                stats.median_ns,
                stats.p95_ns,
                stats.max_ns,
                stats.total_ns
            ),
            (1, 10, 19, 20, 210)
        );
    }
    #[test]
    fn rejects_failed_incorrect_and_inconsistent_runs() {
        let good = RunResult {
            outcome: Outcome::Success,
            stdout: b"BENCHMARK\n".to_vec(),
            stderr: vec![],
            fuel_consumed: 10,
        };
        assert!(check(&good, Some(&good)).is_ok());
        let mut bad = good.clone();
        bad.fuel_consumed += 1;
        assert!(check(&bad, Some(&good)).is_err());
        bad = good.clone();
        bad.stdout.clear();
        assert!(check(&bad, None).is_err());
        bad = good;
        bad.outcome = Outcome::TimeLimit;
        assert!(check(&bad, None).is_err());
    }
    #[test]
    fn bounds_sample_counts_before_execution() {
        for args in [
            ["bench", "fixture", "--samples", "0"],
            ["bench", "fixture", "--samples", "101"],
            ["bench", "fixture", "--warmup", "101"],
        ] {
            assert!(Args::try_parse_from(args).is_err());
        }
    }
}
