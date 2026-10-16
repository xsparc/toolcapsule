# ADR 0001: verified WASI command components

Status: accepted for the initial MVP scope, 2026-10-17.

Use Rust and Wasmtime, a synchronous WASI 0.2 command ABI executed through the
async host, and exact-byte signed manifests. Keep package verification,
execution, evidence, and adapters separate within one crate. This is easier to
embed and distribute than a network service and avoids premature microservices.

WASI 0.3 is stable according to [WASI releases](https://wasi.dev/releases).
Toolchain coverage still favors 0.2; 0.3 is a deliberate future ABI addition.
Wasmtime's [resource controls](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html)
provide fuel and epoch interruption. A host async timeout is also required for
blocked WASI calls, where epoch interruption alone cannot cancel a host future.
The [MCP stdio transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
provides local client interoperability without an exposed HTTP endpoint.

The first profile denies all ambient authority and supplies deterministic clocks
and random fixtures. Generic brokered network and filesystem grants are deferred
because correct isolation, replay, and tenant authorization require separate
design and tests. Unsupported declarations are rejected, never silently ignored.

Consequences: useful pure transformations run now, while tools requiring external
services need future brokers. Replay is scoped to the exact engine/profile and
successful deterministic executions, not arbitrary external side effects.
