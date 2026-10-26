# Roadmap

## TC009: verified preparation cache

After TC008's merge, delegated scope selection prioritizes measured preparation
reuse. ADR 0005 defines bounded LRU retention with current authority checks on hits,
fresh guest state and benchmark validation.

## TC008: shared execution admission

Delegated research/scope selection continues after TC007's merge. ADR 0004 adds
one cloneable, fail-fast pool across callers with atomic batch-window admission
and cancellation-safe reservation ownership. See docs/pool.md.

## TC007: bounded batch execution

The 2026-10-19 next-milestone request continues delegated scope selection and draft
delivery. ADR 0003 adds ordered batches with count/concurrency/byte admission,
fresh guest state and owned cancellation. See docs/batch.md for integration limits.

The initial delivery used three dependent PRs, now merged. Bootstrap main
contained repository metadata; subsequent approved slices are recorded below.

| Slice | Outcome | Acceptance |
| --- | --- | --- |
| TC001 | Signed packages and architecture | Signature, digest, strict-schema, capability and budget admission tests; governance gate |
| TC002 | Constrained WASI runtime and replay | Real component execution, denied I/O, resource exhaustion, fresh state, audit and replay tests |
| TC003 | CLI, MCP, examples and CI | End-to-end signing/execution/replay; MCP lifecycle and error tests; portable CI |

## Next milestone: TC005 execution measurements

The 2026-10-18 request delegates research, next-scope selection, delivery and a
draft PR. ADR 0002 selects a bounded prepared-execution benchmark with separate
phase timings and fixture correctness checks. TC004 was the MCP correction.

Later work requires a bounded accepted scope: WASI 0.3 async ABI; brokered
read-only filesystem and domain-restricted HTTP; external secret handles;
tenant authentication and quotas; signed registry distribution; independent
sandbox review; measured cache and throughput tuning. Subprocess access is
deferred until an independently isolated broker exists.

Ongoing stewardship reviews dependency/security changes, CI failures, open PRs,
and user needs. It produces proposals and bounded reviewable changes. It never
authorizes merges, deployment, releases, paid services, or contacting people.

## TC006: governance validation under optimization

A bounded maintenance correction keeps evidence checks active with Python -O
and -OO. Acceptance covers valid evidence, malformed metadata, missing or
escaping references, broken traceability, deferred-work rationale and freshness.
Synthetic CLI regressions run in the existing platform matrix. The runtime,
capability policy, guest ABI and replay profile are unchanged.

## TC011: trusted audit checkpoints

The 2026-10-22 request continues delegated scope selection and draft delivery.
ADR 0007 closes the retained-chain-head verification gap without depending on
TC010's still-open catalog PR. Acceptance covers anchored prefix/exact verification,
exclusive checkpoint export and valid rewrite/truncation rejection.

## TC012: Signed audit checkpoints

The delegated 2026-10-23 scope continues the merged evidence milestone.
Decision: `docs/decisions/0008-signed-audit-checkpoints.md`.
Acceptance: Round-trip authentication, tampering and wrong-domain rejection, revocation, strict schema and size admission, CLI exclusive output, required trust and prefix/exact verification.

## TC013: Package preflight

The delegated 2026-10-25 scope closes a bounded workflow gap on current main.
Decision: `docs/decisions/0009-package-preflight.md`.
Acceptance: Real WASI fixture identity and profile; malformed source, absent command export and unresolved imports fail; valid signed invalid source distinguishes verify from preflight; trust/digest denial precedes compilation; trapping start and command code are not run.

## TC014: Wasmtime security patch

Pin the current 49.x line to 49.0.4 for the published AArch64 compiler advisory.
Acceptance includes dependency resolution, a new runtime profile, rejection of
legacy-profile replay and the existing isolation, preflight, cache, pool,
checkpoint and governance regressions. The guest ABI and capability policy stay
within the existing design. No Wasmtime 50 migration or new external capability
is included.

## TC015: Explicit signed resource budgets

The delegated 2026-10-26 scope extends an existing admitted contract.
Decision: `docs/decisions/0010-signed-resource-budgets.md`.
Acceptance: Custom limits appear unchanged in authenticated preflight output; input enforcement rejects oversized invocation; stricter host policy rejects the package; unknown/missing/zero/too-small/oversized limits fail before output; default packaging and non-overwrite behavior remain covered.

## TC016: Configuration templates

Delegated 2026-10-27 milestone. Decision: `docs/decisions/0011-configuration-templates.md`.

1. Generate validated limits JSON for `pack --limits`.
2. Generate deny-by-default host-policy JSON, optionally using explicit limits.

Acceptance: Generated limits round-trip through the existing contract. Generated policy denies a signed package until an operator explicitly adds its tool to the allowlist. Invalid and oversized limits produce no output; existing files are never overwritten.
