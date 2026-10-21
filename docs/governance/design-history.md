# Design history

2026-10-19, TC007: the next-milestone request continues delegated scope selection.
ADR 0003 adds a bounded batch API over one prepared trust/policy snapshot. Fresh
guest state, ordered outcomes and owned cancellation preserve the existing runtime
boundary. See batch-direction.md and batch-validation.md.

2026-10-18: delegated next-scope research selects TC005, a bounded synthetic
preparation/reuse benchmark. ADR 0002 preserves runtime and replay behavior.
See benchmark-direction.md and benchmark-validation.md for rationale and evidence.

This index is derived evidence, not a source of permission. Architecture,
roadmap, and ADRs define the intended design; code and tests demonstrate it.

The initial request authorizes creation of a public MIT repository and an MVP
within three PRs. It does not authorize merging, release, deployment, purchases,
or contacting people. Development is agent-assisted with configured Git identity.

Research was performed on 2026-10-17 from the bootstrap baseline. The cited
direction brief informs initial design choices under the requested conceptual
and implementation scope. Future recommendations remain advisory until accepted.

The installed OpenSteward governance checker requires its own project identity.
ToolCapsule uses a repository-local stdlib-only structural/freshness checker
against its own evidence registry. This adaptation does not claim the upstream
OpenSteward checker passed.

2026-10-19: TC006 requires structural validation to run under normal and
optimized Python execution. Explicit conditional guards preserve the existing
sanitized failure categories; optimization must not bypass schema, path or
traceability checks. Freshness remains optional for the static check and is
enforced by the dated strict gate.

2026-10-21: TC010 adds static mcp tool catalog under delegated next-scope selection.
The bounded contract is in docs/decisions/0006-mcp-tool-catalog.md; executed checks and limits are in
docs/governance/catalog-validation.md.
