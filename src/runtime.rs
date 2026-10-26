//! Compiled code may be reused; guest state and authority never are.
use crate::{Error, Manifest, VerifiedPackage};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder, Trap};
use wasmtime_wasi::clocks::{HostMonotonicClock, HostWallClock};
use wasmtime_wasi::p2::{
    bindings::CommandPre,
    pipe::{MemoryInputPipe, MemoryOutputPipe},
};
use wasmtime_wasi::random::Deterministic;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

pub const PROFILE: &str = "toolcapsule-v1-wasmtime-49.0.4-wasip2-fixed-clock-cycle-rng";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    GuestError,
    FuelExhausted,
    TimeLimit,
    Trap,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunResult {
    pub outcome: Outcome,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub fuel_consumed: u64,
}

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
}
impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

struct FixedClock;
impl HostWallClock for FixedClock {
    fn resolution(&self) -> Duration {
        Duration::from_nanos(1)
    }
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
impl HostMonotonicClock for FixedClock {
    fn resolution(&self) -> u64 {
        1
    }
    fn now(&self) -> u64 {
        0
    }
}

pub struct Runtime {
    engine: Engine,
}

/// Preparation evidence only: no instance is created and no guest code is run.
#[derive(Debug, Serialize)]
pub struct PreflightReport {
    pub schema_version: u32,
    pub profile: String,
    pub manifest: Manifest,
    pub manifest_sha256: String,
    pub signer: String,
    pub compiled: bool,
    pub executed: bool,
}

impl Runtime {
    /// Check compilation, imports and the command interface after package admission.
    /// Compilation CPU/RSS still require an externally bounded worker.
    pub fn preflight(&self, package: VerifiedPackage) -> Result<PreflightReport, Error> {
        let tool = self.prepare(package)?;
        Ok(PreflightReport {
            schema_version: 1,
            profile: PROFILE.into(),
            manifest: tool.manifest().clone(),
            manifest_sha256: tool.manifest_sha256().into(),
            signer: tool.signer().into(),
            compiled: true,
            executed: false,
        })
    }

    pub fn new() -> Result<Self, Error> {
        let mut config = Config::new();
        config
            .consume_fuel(true)
            .wasm_component_model(true)
            .wasm_multi_memory(false)
            .wasm_memory64(false)
            .wasm_relaxed_simd(false)
            .cranelift_nan_canonicalization(true)
            .wasm_backtrace_max_frames(None);
        Ok(Self {
            engine: Engine::new(&config).map_err(|_| Error::Runtime)?,
        })
    }

    /// Compilation belongs in a bounded worker admission path, outside request deadlines.
    pub fn prepare(&self, package: VerifiedPackage) -> Result<PreparedTool, Error> {
        // Component::new accepts only source bytes; no native artifact deserialization.
        let component =
            Component::new(&self.engine, package.component()).map_err(|_| Error::Component)?;
        let mut linker = Linker::<State>::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).map_err(|_| Error::Runtime)?;
        let pre = CommandPre::new(
            linker
                .instantiate_pre(&component)
                .map_err(|_| Error::Component)?,
        )
        .map_err(|_| Error::Component)?;
        Ok(PreparedTool {
            engine: self.engine.clone(),
            pre,
            package,
        })
    }
}

pub struct PreparedTool {
    engine: Engine,
    pre: CommandPre<State>,
    package: VerifiedPackage,
}

impl PreparedTool {
    pub fn manifest(&self) -> &Manifest {
        self.package.manifest()
    }
    pub fn manifest_sha256(&self) -> &str {
        self.package.manifest_sha256()
    }
    pub fn signer(&self) -> &str {
        self.package.signer()
    }

    /// Invoke inside a Tokio runtime with its timer enabled. Seed is public fixture data.
    pub async fn run(&self, input: &[u8], seed: u64) -> Result<RunResult, Error> {
        let limits = &self.manifest().limits;
        if input.len() > limits.input_bytes {
            return Err(Error::InputLimit);
        }
        let stdout = MemoryOutputPipe::new(limits.output_bytes);
        let stderr = MemoryOutputPipe::new(limits.output_bytes);
        let mut builder = WasiCtx::builder();
        builder
            .stdin(MemoryInputPipe::new(input.to_vec()))
            .stdout(stdout.clone())
            .stderr(stderr.clone())
            .allow_tcp(false)
            .allow_udp(false)
            .allow_ip_name_lookup(false)
            .socket_addr_check(|_, _| Box::pin(async { false }))
            .wall_clock(FixedClock)
            .monotonic_clock(FixedClock)
            .secure_random(Deterministic::new(seed.to_le_bytes().to_vec()))
            .insecure_random(Deterministic::new(seed.to_be_bytes().to_vec()))
            .insecure_random_seed(seed as u128)
            .max_random_size(limits.output_bytes as u64);
        let mut table = ResourceTable::new();
        table.set_max_capacity(256);
        let state = State {
            ctx: builder.build(),
            table,
            limits: StoreLimitsBuilder::new()
                .memory_size(limits.memory_bytes)
                .memories(1)
                .table_elements(10_000)
                .tables(4)
                .instances(32)
                .trap_on_grow_failure(true)
                .build(),
        };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| &mut s.limits);
        store.set_fuel(limits.fuel).map_err(|_| Error::Runtime)?;
        // Cooperative yields let Tokio enforce deadlines even for a pure Wasm loop.
        // Async WASI host calls are covered by the same timeout future.
        store
            .fuel_async_yield_interval(Some(10_000))
            .map_err(|_| Error::Runtime)?;
        let execution = async {
            let command = self.pre.instantiate_async(&mut store).await?;
            command.wasi_cli_run().call_run(&mut store).await
        };
        let outcome =
            match tokio::time::timeout(Duration::from_millis(limits.timeout_ms), execution).await {
                Err(_) => Outcome::TimeLimit,
                Ok(Ok(Ok(()))) => Outcome::Success,
                Ok(Ok(Err(()))) => Outcome::GuestError,
                Ok(Err(error)) => {
                    if error.downcast_ref::<Trap>() == Some(&Trap::OutOfFuel) {
                        Outcome::FuelExhausted
                    } else if let Some(exit) = error.downcast_ref::<wasmtime_wasi::I32Exit>() {
                        if exit.0 == 0 {
                            Outcome::Success
                        } else {
                            Outcome::GuestError
                        }
                    } else {
                        Outcome::Trap
                    }
                }
            };
        let fuel_consumed = limits.fuel - store.get_fuel().map_err(|_| Error::Runtime)?;
        Ok(RunResult {
            outcome,
            stdout: stdout.contents().to_vec(),
            stderr: stderr.contents().to_vec(),
            fuel_consumed,
        })
    }
}
