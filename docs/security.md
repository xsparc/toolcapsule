# Security boundary

Treat components, manifests, MCP input, and guest output as untrusted.
A valid signature identifies an operator-trusted publisher; it does not make a
component safe. Trust configuration is supplied separately from the package.
Policy defaults deny every tool. Manifest fields cannot grant host authority.

The development key generator writes an unencrypted seed. Unix files are created
with mode 0600; Windows files inherit directory ACLs. Use a protected directory
and external signing/key management for operational publishers. Runtime workers
only need the public trust store. Key IDs and trust-store updates are operator-owned.

No filesystem preopens, environment inheritance, host stdio inheritance,
network permission, secrets, or subprocess APIs are supplied to guests. Inputs
and outputs are bounded. Fixed clocks and seeded random are fixtures, not a
cryptographic entropy service. Guest output can contain prompt injection and
must remain untrusted data in consuming agents.

Fuel, linear-memory, table, instance, input/output and wall-time controls reduce
guest abuse. Each invocation permits one linear memory, four
tables of at most 10,000 elements each, 32 core instances, and 256 WASI resource
handles. Stdout and stderr each have the declared output byte limit.
Wasmtime/compiler vulnerabilities and host compromise are outside this boundary.
Compilation is not constrained by execution fuel or
deadline. Run untrusted publishers in separately resource-limited worker
processes. OS sandboxing and per-tenant worker admission are deployment duties.

Audit hashes detect modifications only when an earlier chain head is retained
in trusted storage. An attacker able to replace the whole log can rewrite it.
Logs are not signed or externally anchored. Raw replay inputs are caller-owned
sensitive data; they are not stored in the default audit record. Tool names and
signer IDs are public package metadata. Logs can still reveal usage patterns.

Clock readings are fixed, but WASI timer subscriptions still use the host async
scheduler. Timer readiness and execution deadlines are not a virtual-time trace.
Timing-sensitive guests may fail replay even after a successful original run;
the verifier reports a mismatch. Guaranteed replay across host scheduling or
runtime versions is not claimed. Prefer pure transformations without timer races.

Report security concerns through GitHub private vulnerability reporting when
enabled; otherwise open an issue without exploit details or secrets.

The optional trusted-checkpoint verifier checks a supplied retained prefix and
all subsequent entries. Checkpoints are unsigned; protect them independently.
It does not detect removal after the anchor unless exact mode is used. See
[audit checkpoints](audit-checkpoints.md) for empty-anchor and snapshot limits.

Optional signed audit checkpoints authenticate retained anchors using a separately
selected host trust store. Verification still requires independently retained
freshness evidence; signatures do not prevent replay of older valid checkpoints.
See [checkpoint custody](audit-checkpoints.md).

## Runtime security patch, 2026-10-26

Use Wasmtime/wasmtime-wasi 49.0.4. The previous 49.0.3 pin is affected by
[RUSTSEC-2026-0379](https://rustsec.org/advisories/RUSTSEC-2026-0379.html):
Cranelift on AArch64 can miscompile a large branch table and permit a sandbox
escape. The [upstream 49.0.4 release](https://github.com/bytecodealliance/wasmtime/releases/tag/v49.0.4)
contains the patch. Guest capabilities and signatures do not replace an engine
security update.

The runtime profile now identifies 49.0.4. Preserve old receipts and audit
checkpoints as historical evidence; do not relabel them with the new profile.
Replay rejects 49.0.3 receipts. Rebuild and restart integrating workers so old
prepared tools, cache entries and retained handles cannot keep the old engine
alive. This change does not migrate persisted evidence or deploy workers.
