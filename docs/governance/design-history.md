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

2026-10-22: TC011 adds trusted audit checkpoint verification under delegated scope selection.
See docs/decisions/0007-trusted-audit-checkpoints.md for acceptance and
docs/governance/checkpoint-validation.md for actual checks and limits.

## 2026-10-23: TC012

Signed audit checkpoints extends the merged evidence workflow. See the accepted decision at
`docs/decisions/0008-signed-audit-checkpoints.md` and validation at `docs/governance/signed-checkpoint-validation.md`.

## 2026-10-25: TC013

Package preflight: accepted decision `docs/decisions/0009-package-preflight.md`;
verification record `docs/governance/preflight-validation.md`.

## 2026-10-26: TC015

Explicit signed resource budgets: decision `docs/decisions/0010-signed-resource-budgets.md`;
verification `docs/governance/budget-validation.md`.

## 2026-10-27: TC016

Configuration templates: decision `docs/decisions/0011-configuration-templates.md`; validation `docs/governance/template-validation.md`.

## 2026-10-27: TC017

Offline configuration validation: decision `docs/decisions/0012-configuration-validation.md`; validation `docs/governance/configuration-validation.md`.

## 2026-10-28: TC018

Trust validation and budget-fit reporting: decision `docs/decisions/0013-trust-budget-checks.md`; validation `docs/governance/trust-budget-validation.md`.

## 2026-10-29: TC019

Validation diagnostics and configuration acceptance: decision docs/decisions/0014-validation-diagnostics.md; validation docs/governance/validation-diagnostics-validation.md.

## 2026-10-31: TC020

Configuration contracts: decision `docs/decisions/0015-configuration-contracts.md`; verification `docs/governance/configuration-contracts-validation.md`.
