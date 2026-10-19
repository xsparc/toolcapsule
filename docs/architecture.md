# Architecture

ToolCapsule is an embeddable Rust library with thin CLI and MCP adapters.
The initial contract is a WASI 0.2 command: UTF-8 input on stdin, bounded stdout
and stderr, and an exit result. Wasmtime is pinned to stable 49.0.3 in the
execution slice. WASI 0.3 async integration is a subsequent compatibility slice.

## Trust and execution boundaries

1. The operator supplies a trust store, tool allowlist, and maximum budgets.
2. A bounded envelope carries exact manifest bytes and an Ed25519 signature
   over a domain prefix and those bytes. The manifest binds the component SHA-256.
3. Verification precedes compilation. Unknown fields, unsupported ABI versions,
   unsupported capabilities, and requests exceeding host policy fail closed.
4. A prepared component retains its verified bytes and compiled code. Each
   invocation gets a fresh store, WASI context, memory, fuel, and captured I/O.
5. Execution produces a structured result. Audit records contain digests and
   outcomes, never raw inputs, outputs, secrets, or host paths.
6. Replay re-executes with the supplied input and fixture seed, checking identity,
   engine profile, outcome, fuel, and output digests against recorded evidence.

The MVP capability profile explicitly declares filesystem, network, secrets,
subprocess, clocks, and randomness. Only empty host grants, fixed clocks, and
seeded randomness are admitted. Deterministic random bytes are unsuitable for
cryptography. Packages must opt into this profile in their signed manifest.

## Interoperability and scaling

The bounded batch API reuses one prepared object with at most 16 active invocations
and returns ordered results for up to 64 inputs. Aggregate input and retained-output
admission precedes execution. Limits are per batch, not global quotas; external
worker memory limits remain necessary. See docs/batch.md and ADR 0003.

JSON contracts are versioned and language-neutral. MCP stdio exposes verified
tools to agent clients without embedding an agent framework. The library is the
integration point for SaaS queue workers. A worker may reuse prepared code while
creating fresh state for each invocation. No global mutable tenant trust store,
disk cache, or implicit package download exists.

Prepared tools capture a trust/policy snapshot. The service must replace prepared
instances after revoking a signer or changing tenant policy; no background trust
refresh occurs in the execution library.

SaaS integrators must authenticate callers before selecting tenant-specific
trust and policy, bound queue length and parallelism, isolate workers in OS
processes/containers, and export audit records to durable tenant-specific
storage. This MVP does not provide HTTP authentication, billing, remote package
fetching, or a hosted control plane. One MCP process serves one configured tool.
Multiple stateless processes scale independently; no database is required.

The component byte cap bounds intake, not JIT CPU/RSS. Fuel bounds guest
computation, memory limits bound guest linear memory, and an async deadline
bounds guest execution including blocked host calls. Compilation and total
process memory need external worker limits. Public multi-tenant deployment is
not claimed.

## Portability and performance

Rust and Wasmtime support Windows, Linux, and macOS. CI exercises all three.
Only source Wasm components are accepted; native serialized code is never
deserialized. Prepared code reuse avoids repeated compilation; guest state is
never reused. Engine upgrades require replay-profile changes and regression
tests. Performance claims require measured workloads; no throughput SLA is set.
The synthetic benchmark in docs/benchmark.md measures verification, preparation
and fresh-state invocation separately, without changing the runtime profile.
