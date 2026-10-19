# ADR 0003: Bounded batches of prepared invocations

Status: accepted under delegated next-milestone selection, 2026-10-19.

Baseline: a578b46. TC007 builds on the preparation benchmark with a small embedding
API for bounded concurrency. One prepared policy/trust snapshot serves a batch;
fresh stores preserve isolation. Validate count, concurrency, aggregate input and
worst-case retained output before execution. Return results in input order.
JoinSet owns tasks; cancellation aborts active work and never detaches queued work.
Infrastructure failures await cleanup; guest outcomes remain per invocation.

Acceptance: bounded scheduler, reverse completion ordering, cancellation cleanup,
resource overflow rejection, 64 fresh-state executions, mixed real WASI outcomes
and receipt replay, plus required format/Clippy/tests/governance. Expected paths:
batch module/example, runtime tests and traceability. Risks: confusing per-call
limits with global quotas, cooperative cleanup, multiplied guest memory and queue
wait outside invocation deadlines. No new capability, ABI, pool or native cache.
See docs/governance/batch-direction.md and docs/batch.md.
