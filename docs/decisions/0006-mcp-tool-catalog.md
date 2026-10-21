# ADR 0006: static MCP tool catalog

Status: accepted

Date: 2026-10-21

TC010 follows verified preparation reuse under delegated next-scope selection.
Serve 1..16 explicitly listed local packages in one MCP process. A strict version-1
catalog names manifest and component paths relative to the catalog file. Shared
operator-supplied trust and policy admit every package before any compilation or
stdio response. Reject duplicate signed tool names and more than 64 MiB aggregate
source bytes. Compile source components with one runtime; never load native code.

List tools in lexical name order, eight per page, using a static continuation
cursor. Dispatch calls by exact signed name, preserving per-tool limits, fresh
state and digest-only receipts in the shared audit chain. Existing single-tool
CLI and Server::new remain compatible. Server::with_tools also enforces count and
unique names for embedding callers; those callers own preparation admission.

Acceptance: real stdio pagination, invalid cursors, named execution and distinct
receipt identities; duplicate/empty/oversized/unknown-field catalog rejection;
current trust/policy/digest admission and aggregate-byte rejection before compile;
existing lifecycle, replay and runtime tests remain passing.

The catalog is an authority snapshot until process restart. No live discovery,
fetching, registration, new guest capabilities, tenant authentication, MCP protocol
upgrade or parallel stdio execution is added. The source cap does not bound JIT
RSS or CPU; use external process limits. The existing shared audit file size cap
and checkpoint requirements continue to apply.
