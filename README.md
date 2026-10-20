# ToolCapsule

Repeated package preparation can use the [verified cache](docs/cache.md).

Worker integrations can share [execution admission](docs/pool.md) across callers.

Run AI-agent tools as signed WebAssembly components with explicit capabilities,
resource budgets, audit receipts, and deterministic replay. Use the Rust library,
CLI, or MCP stdio adapter. MIT licensed; no hosted service is required.

The MVP runs **WASI 0.2 command components** on pinned Wasmtime 49.0.3. It admits
pure transformations: no host files, environment, sockets, secrets, or
subprocesses. Unsupported grants fail closed. Clocks are fixed and random bytes
are seeded fixtures. WASI 0.3 and brokered external capabilities are on the
[roadmap](docs/roadmap.md).

## Quick start

Install Rust 1.96+ and Python 3.11+, then run from a checkout:

```sh
rustup target add wasm32-wasip2
python scripts/demo.py
```

The demo builds a real WASI uppercase tool and the CLI, generates a local key,
signs the package, verifies it, executes it, and replays its receipt. Expect
`HELLO TOOLCAPSULE` and `{"matched":true}`. Each demo uses a new ignored directory
under `.local`; private keys and replay data stay there. No remote services run.

Build the standalone CLI with `cargo build --release --locked`, or install it
with `cargo install --path . --locked`. Use `toolcapsule --help` for commands.

## Package and run a tool

Compile your command to `wasm32-wasip2`. It reads stdin, writes stdout/stderr,
and returns an exit status. The component path is supplied separately and never
taken from an untrusted manifest.

```sh
toolcapsule keygen --key signing.key --trust trust.json
toolcapsule pack --component tool.wasm --name uppercase --key signing.key --out signed.json
toolcapsule verify --manifest signed.json --component tool.wasm --trust trust.json --policy examples/policy.json
echo hello | toolcapsule run --manifest signed.json --component tool.wasm --trust trust.json --policy examples/policy.json --audit audit.jsonl --receipt receipt.json
echo hello | toolcapsule replay --manifest signed.json --component tool.wasm --trust trust.json --policy examples/policy.json --receipt receipt.json
toolcapsule audit-verify --audit audit.jsonl
```

Keep signing keys separate from runtime workers. The public trust store maps a
key ID to its Ed25519 public key; obtain it through an operator-trusted channel.
The [example policy](examples/policy.json) explicitly allows only `uppercase`.

Embedding multiple invocations? The [bounded batch API](docs/batch.md) reuses one
prepared tool with ordered results, byte/concurrency limits and cancellation.
The Rust API permits custom signed limits; `pack` uses the default profile:
10 million fuel, 64 MiB guest memory, one second, 64 KiB input, and 64 KiB each
for stdout and stderr. Exceeding host policy is rejected, never silently clamped.

`run` prints JSON and returns exit code 0 on success, 1 on guest failure, and 2
on configuration, verification, I/O, or replay errors. `verify` checks trust and
policy; compilation/interface validation occurs when preparing execution.
Audit records hold digests, not raw I/O. CLI text rendering replaces invalid
UTF-8; the Rust API and receipt digests preserve the original bytes.

## Connect an MCP client

Launch `toolcapsule mcp` with the same four package options and `--audit`.
Configure your client with the executable and these arguments, resolving paths
against the client's working directory:

```json
{
  "mcpServers": {
    "toolcapsule": {
      "command": "toolcapsule",
      "args": ["mcp", "--manifest", "signed.json", "--component", "tool.wasm",
        "--trust", "trust.json", "--policy", "policy.json", "--audit", "audit.jsonl"]
    }
  }
}
```

The adapter supports MCP `2025-11-25`, initialization, ping, `tools/list`, and
`tools/call`. Arguments are `{"input":"hello","seed":42}`; seed is optional.
It serves one tool sequentially per process, limits messages to 1 MiB, and exits
on oversized or truncated frames. Calls have the signed execution deadline.
Client cancellation is not processed while a call is running. Guest output
remains untrusted text. Binary output is available through the library; MCP
requires UTF-8. Protocol traffic alone goes to stdout.

## Embed in SaaS workers

Verify a package using tenant-selected `TrustStore` and `Policy`, prepare it once
with `Runtime::prepare`, and call `PreparedTool::run` inside a Tokio runtime.
Each call creates fresh guest state. Persist `Receipt` records through your own
tenant-aware sink or the provided locked JSONL writer. The local writer verifies
the existing chain and caps each file at 8 MiB; rotate files and retain trusted
checkpoints outside the worker. Replay verifies successful runs against the
same engine profile, exact package, input, seed, fuel, and output digests.
Timer readiness still follows host scheduling; timing-sensitive tools can fail
replay. This is verified re-execution, not an external-event recording system.

Authentication, queue admission, concurrency quotas, total process memory, JIT
compilation limits, and durable audit storage belong to the integrating service.
Use separately constrained OS workers for untrusted publishers. This MVP is not
an independently audited multi-tenant security boundary. See
[architecture](docs/architecture.md) and [security](docs/security.md).

## Development and stewardship

See [development](docs/development.md), [verification evidence](docs/governance/verification.md),
and [design history](docs/governance/design-history.md). CI tests Windows, Linux,
and macOS. The initial delivery consists of three dependent PRs: package trust,
runtime/replay, and CLI/MCP. Review and merge them in order, retargeting each next
PR to `main` after its dependency lands.

OpenSteward guides research, bounded implementation, and evidence maintenance.
Development is agent-assisted. Future improvements require review; stewardship
does not authorize merging, deployment, releases, or paid services.
