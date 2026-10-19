# DirectionBriefV1: bounded prepared invocation batches

- schema_version: 1
- baseline_commit: a578b46
- scanned_at: 2026-10-19
- coverage: focused prepared-tool integration, task ownership and cancellation.

1. finding: Tokio JoinSet yields completion order and aborts owned tasks on drop.
   [Tokio API](https://docs.rs/tokio/1.53.3/tokio/task/struct.JoinSet.html).
   source_event_date: version 1.53.3 documentation, accessed 2026-10-19;
   confidence: high; relevance: bounded invocation ownership;
   direction_alignment: reusable execution; opportunity_or_risk: explicit ordering
   and cooperative cleanup are required; engineering_cost: medium; disposition:
   adopt; roadmap_delta: TC007; verification_needed: active-window counts, ordering,
   errors and dropped-call cleanup against the pinned dependency.

Overall recommendation: add a bounded batch embedding API before throughput tuning.
The previous measurement established preparation/reuse as distinct costs; this
slice makes reuse convenient without adding capabilities. Selection and draft PR
delivery follow the continuing delegated next-milestone request. Evidence gaps:
global tenant quotas, process RSS, multi-thread throughput and independent review.
