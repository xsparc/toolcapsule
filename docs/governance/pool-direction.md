# DirectionBriefV1: shared execution admission

- schema_version: 1
- baseline_commit: 92ccdd5
- scanned_at: 2026-10-19
- coverage: merged TC007, runtime/batch ownership, roadmap and Tokio admission API.

Finding: independent batches each admit 16 invocations; there is no shared ceiling.
[Tokio Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html)
provides atomic nonblocking owned reservations. Keeping reservation ownership in
spawned tasks avoids releasing capacity before cancellation cleanup finishes.
source_event_date: undated living documentation, accessed 2026-10-19;
confidence: high; relevance: concurrent worker callers;
direction_alignment: bounded execution; opportunity_or_risk: aggregate overload;
engineering_cost: medium; disposition: adopted under delegated scope selection;
roadmap_delta: TC008 process-local admission; verification_needed: saturation,
atomic batch admission, cancellation ownership, fresh guest state and replay.

Recommendation: add a cloneable pool with no waiting queue. Select this foundation
before tenant quotas or an HTTP service. WASI 0.3 and brokered I/O remain separate
compatibility/security designs; neither resolves cross-batch overload. Gaps:
this is not authentication, distributed admission, memory accounting or rate limiting.

The maintainer delegated research/scope selection and requested the next milestone
and draft PR. No merge, deployment or release is included.
