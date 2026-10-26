# DirectionBriefV1: Explicit signed resource budgets

- schema_version: 1
- baseline_commit: 2b3649645104a64a9150ef02ab0bdd3b20e6bca5
- scanned_at: 2026-10-26
- coverage: roadmap, architecture, current CLI and runtime/planner contracts,
  recent merges, open draft status, existing tests and primary platform guidance.

Finding: The runtime already enforces signed limits, but CLI packaging hardcodes defaults. Exposing the existing closed contract lets publishers select smaller budgets without constructing envelopes outside the CLI. Wasmtime treats fuel as configured guest execution accounting; this does not replace host memory and deadline controls.

Source: [Wasmtime fuel documentation](https://docs.wasmtime.dev/api/wasmtime/struct.Store.html#method.set_fuel).
source_event_date: undated living documentation, accessed 2026-10-26;
confidence: high, checked against local pinned-runtime regressions;
relevance: explicit signed resource budgets; direction_alignment: explicit bounded work;
opportunity_or_risk: clearer workload control without implicit authority;
engineering_cost: low; disposition: adopt; roadmap_delta: TC015;
verification_needed: Custom limits appear unchanged in authenticated preflight output; input enforcement rejects oversized invocation; stricter host policy rejects the package; unknown/missing/zero/too-small/oversized limits fail before output; default packaging and non-overwrite behavior remain covered.

Overall recommendation: implement the bounded existing-contract extension.
The open security patch #18 addresses the
[published AArch64 advisory](https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-mghf-wjfp-p5xm)
and has passing three-platform CI. This branch uses its patched 49.0.4 baseline;
it does not duplicate or merge that patch. New ABI or broker capabilities remain
larger independent scopes.

Evidence gaps: A publisher requests budgets; it cannot grant host capabilities or increase the operator's policy. Fuel is engine-specific work accounting, not a portable wall-time estimate. Guest memory, fuel, I/O and timeout limits do not bound compilation CPU/RSS or all process allocations. No execution ABI or receipt profile changes are introduced by this slice. It is stacked on security patch PR #18 and must follow that patch.

