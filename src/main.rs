use clap::{Args, Parser, Subcommand};
use ed25519_dalek::SigningKey;
use std::{
    io::{self, Write},
    path::PathBuf,
};
use toolcapsule::{
    audit::{AuditCheckpoint, Receipt},
    checkpoint_signing::{MAX_CHECKPOINT_BYTES, MAX_SIGNED_CHECKPOINT_BYTES, SignedCheckpoint},
    mcp::{MAX_FRAME_BYTES, Server},
    package::{MAX_ENVELOPE_BYTES, digest},
    runtime::{Outcome, PreparedTool, Runtime},
    storage::*,
    *,
};

#[derive(Parser)]
#[command(
    version,
    about = "Run signed WASI tools with explicit authority and budgets"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a signing seed and a separate public trust store. Never overwrites files.
    Keygen {
        #[arg(long)]
        key: PathBuf,
        #[arg(long)]
        trust: PathBuf,
        #[arg(long, default_value = "local")]
        key_id: String,
    },
    /// Create a signed manifest with default or explicitly supplied resource limits.
    Pack {
        #[arg(long)]
        component: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "0.1.0")]
        version: String,
        #[arg(long, default_value = "A sandboxed WASI tool")]
        description: String,
        /// Closed Limits JSON (at most 4 KiB); requests still require host admission.
        #[arg(long)]
        limits: Option<PathBuf>,
        #[arg(long)]
        key: PathBuf,
        #[arg(long, default_value = "local")]
        key_id: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Verify package trust and policy without compiling or executing it.
    Verify {
        #[command(flatten)]
        package: PackageArgs,
    },
    /// Verify, compile and check the WASI command interface without running guest code.
    Preflight {
        #[command(flatten)]
        package: PackageArgs,
    },
    /// Execute stdin once, emit structured JSON, and append a digest-only audit record.
    Run {
        #[command(flatten)]
        package: PackageArgs,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long)]
        audit: PathBuf,
        #[arg(long)]
        receipt: Option<PathBuf>,
    },
    /// Re-execute stdin against a successful receipt from the same runtime profile.
    Replay {
        #[command(flatten)]
        package: PackageArgs,
        #[arg(long)]
        receipt: PathBuf,
    },
    /// Serve one verified tool over MCP stdio. Stdout contains only JSON-RPC.
    Mcp {
        #[command(flatten)]
        package: PackageArgs,
        #[arg(long)]
        audit: PathBuf,
    },
    /// Validate an audit chain and print its entry count and checkpoint hash.
    AuditVerify {
        #[arg(long)]
        audit: PathBuf,
        /// Previously retained trusted checkpoint; require this prefix to remain intact.
        #[arg(long)]
        checkpoint: Option<PathBuf>,
        /// Reject appended entries too. Requires an unsigned or signed checkpoint.
        #[arg(long)]
        exact: bool,
        /// Authenticate a checkpoint with the explicitly supplied checkpoint trust store.
        #[arg(long, conflicts_with = "checkpoint", requires = "trust")]
        signed_checkpoint: Option<PathBuf>,
        #[arg(long, requires = "signed_checkpoint")]
        trust: Option<PathBuf>,
    },
    /// Verify a log and write a new unsigned checkpoint for protected retention.
    AuditCheckpoint {
        #[arg(long)]
        audit: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Sign an explicit checkpoint offline. Never overwrites the destination.
    AuditCheckpointSign {
        #[arg(long)]
        checkpoint: PathBuf,
        #[arg(long)]
        key: PathBuf,
        #[arg(long, default_value = "local")]
        key_id: String,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Args)]
struct PackageArgs {
    #[arg(long)]
    manifest: PathBuf,
    #[arg(long)]
    component: PathBuf,
    #[arg(long)]
    trust: PathBuf,
    #[arg(long)]
    policy: PathBuf,
}

impl PackageArgs {
    fn verify(&self) -> Result<VerifiedPackage, Error> {
        let trust: TrustStore = read_json(&self.trust, 64 * 1024)?;
        let policy: Policy = read_json(&self.policy, 64 * 1024)?;
        let bytes = read_file(
            &self.component,
            policy.max_component_bytes.min(64 * 1024 * 1024),
        )?;
        VerifiedPackage::verify(
            &read_file(&self.manifest, MAX_ENVELOPE_BYTES)?,
            bytes,
            &trust,
            &policy,
        )
    }
    fn prepare(&self) -> Result<PreparedTool, Error> {
        Runtime::new()?.prepare(self.verify()?)
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<(), Error> {
    let mut out = io::stdout().lock();
    serde_json::to_writer(&mut out, value).map_err(|_| Error::Io)?;
    out.write_all(b"\n")
        .and_then(|()| out.flush())
        .map_err(|_| Error::Io)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match execute(Cli::parse()).await {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::FAILURE,
        Err(error) => {
            eprintln!("toolcapsule: {error}");
            std::process::ExitCode::from(2)
        }
    }
}

async fn execute(cli: Cli) -> Result<bool, Error> {
    match cli.command {
        Command::Keygen { key, trust, key_id } => {
            if key_id.is_empty() || key_id.len() > 64 {
                return Err(Error::Format);
            }
            // Fail before creating either file when a destination already exists.
            if key == trust || key.exists() || trust.exists() {
                return Err(Error::Io);
            }
            let mut bytes = [0u8; 32];
            getrandom::fill(&mut bytes).map_err(|_| Error::Runtime)?;
            let signing = SigningKey::from_bytes(&bytes);
            let keys: TrustStore =
                [(key_id, hex::encode(signing.verifying_key().to_bytes()))].into();
            write_new(&key, hex::encode(bytes).as_bytes())?;
            write_new(
                &trust,
                &serde_json::to_vec_pretty(&keys).map_err(|_| Error::Format)?,
            )?;
        }
        Command::Pack {
            component,
            name,
            version,
            description,
            limits,
            key,
            key_id,
            out,
        } => {
            let limits: Limits = match limits {
                Some(path) => read_json(&path, 4096)?,
                None => Limits::default(),
            };
            limits.validate()?;
            let key = read_file(&key, 128)?;
            let key: [u8; 32] =
                hex::decode(std::str::from_utf8(&key).map_err(|_| Error::Format)?.trim())
                    .map_err(|_| Error::Format)?
                    .try_into()
                    .map_err(|_| Error::Format)?;
            let manifest = Manifest {
                schema_version: 1,
                name,
                version,
                description,
                abi: "wasi:cli/command@0.2".into(),
                component_sha256: digest(&read_file(&component, 16 * 1024 * 1024)?),
                capabilities: Capabilities::default(),
                limits,
            };
            let signed = SignedManifest::sign(&manifest, &key_id, &SigningKey::from_bytes(&key))?;
            write_new(
                &out,
                &serde_json::to_vec_pretty(&signed).map_err(|_| Error::Format)?,
            )?;
        }
        Command::Verify { package } => {
            let package = package.verify()?;
            print_json(
                &serde_json::json!({"verified":true,"name":package.manifest().name,"manifest_sha256":package.manifest_sha256()}),
            )?;
        }
        Command::Preflight { package } => {
            let package = package.verify()?;
            print_json(&Runtime::new()?.preflight(package)?)?;
        }
        Command::Run {
            package,
            seed,
            audit,
            receipt,
        } => {
            let tool = package.prepare()?;
            let input = read_bounded(io::stdin().lock(), tool.manifest().limits.input_bytes)?;
            let result = tool.run(&input, seed).await?;
            let record = Receipt::new(&tool, &input, seed, &result);
            append_audit(&audit, record.clone())?;
            if let Some(path) = receipt {
                write_new(
                    &path,
                    &serde_json::to_vec_pretty(&record).map_err(|_| Error::Format)?,
                )?;
            }
            print_json(
                &serde_json::json!({"outcome":result.outcome,"stdout":String::from_utf8_lossy(&result.stdout),
                "stderr":String::from_utf8_lossy(&result.stderr),"fuel_consumed":result.fuel_consumed,"receipt":record}),
            )?;
            return Ok(result.outcome == Outcome::Success);
        }
        Command::Replay { package, receipt } => {
            let tool = package.prepare()?;
            let receipt: Receipt = read_json(&receipt, 64 * 1024)?;
            let input = read_bounded(io::stdin().lock(), tool.manifest().limits.input_bytes)?;
            receipt.replay(&tool, &input).await?;
            print_json(&serde_json::json!({"matched":true}))?;
        }
        Command::Mcp { package, audit } => {
            let mut server = Server::new(package.prepare()?, audit);
            let mut input = io::stdin().lock();
            while let Some(frame) = read_frame(&mut input, MAX_FRAME_BYTES)? {
                if let Some(response) = server.handle(&frame).await {
                    print_json(&response)?;
                }
            }
        }
        Command::AuditVerify {
            audit,
            checkpoint,
            exact,
            signed_checkpoint,
            trust,
        } => {
            if exact && checkpoint.is_none() && signed_checkpoint.is_none() {
                return Err(Error::Format);
            }
            let bytes = read_file(&audit, MAX_AUDIT_BYTES)?;
            let anchor = match (signed_checkpoint, trust) {
                (Some(path), Some(trust)) => Some(SignedCheckpoint::verify(
                    &read_file(&path, MAX_SIGNED_CHECKPOINT_BYTES)?,
                    &read_json(&trust, 64 * 1024)?,
                )?),
                _ => None,
            };
            let (entries, checkpoint) = match (anchor, checkpoint) {
                (Some(anchor), _) => verify_log_against(&bytes, &anchor, exact)?,
                (None, Some(path)) => {
                    let anchor: AuditCheckpoint = read_json(&path, MAX_CHECKPOINT_BYTES)?;
                    verify_log_against(&bytes, &anchor, exact)?
                }
                (None, None) => verify_log(&bytes)?,
            };
            print_json(&serde_json::json!({"entries":entries,"checkpoint":checkpoint}))?;
        }
        Command::AuditCheckpoint { audit, out } => {
            let checkpoint = checkpoint_log(&read_file(&audit, MAX_AUDIT_BYTES)?)?;
            write_new(
                &out,
                &serde_json::to_vec_pretty(&checkpoint).map_err(|_| Error::Audit)?,
            )?;
            print_json(&checkpoint)?;
        }
        Command::AuditCheckpointSign {
            checkpoint,
            key,
            key_id,
            out,
        } => {
            let checkpoint: AuditCheckpoint = read_json(&checkpoint, MAX_CHECKPOINT_BYTES)?;
            let bytes = read_file(&key, 128)?;
            let seed: [u8; 32] = hex::decode(
                std::str::from_utf8(&bytes)
                    .map_err(|_| Error::Format)?
                    .trim(),
            )
            .map_err(|_| Error::Format)?
            .try_into()
            .map_err(|_| Error::Format)?;
            let signed =
                SignedCheckpoint::sign(&checkpoint, &key_id, &SigningKey::from_bytes(&seed))?;
            write_new(
                &out,
                &serde_json::to_vec_pretty(&signed).map_err(|_| Error::Format)?,
            )?;
            print_json(
                &serde_json::json!({"signed":true,"key_id":key_id,"entries":checkpoint.entries}),
            )?;
        }
    }
    Ok(true)
}
