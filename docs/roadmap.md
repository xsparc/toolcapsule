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
