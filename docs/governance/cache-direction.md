# DirectionBriefV1: verified preparation reuse

- schema_version: 1
- baseline_commit: 617cb47
- scanned_at: 2026-10-20
- coverage: TC008 merge, roadmap scaling candidates, recorded preparation timings,
  runtime trust ownership and current Wasmtime instantiation guidance.

Finding: the existing fixture measured about 196 ms median preparation versus
1.57 ms verification. [Wasmtime fast instantiation](https://docs.wasmtime.dev/examples-fast-instantiation.html)
describes reuse of compiled/pre-instantiated code. The project already uses
CommandPre, but callers must retain and organize prepared tools themselves.
source_event_date: undated living documentation, accessed 2026-10-20;
confidence: high; relevance: repeated signed package preparation;
direction_alignment: measured cache tuning; opportunity_or_risk: avoid compilation
without serving revoked authority; engineering_cost: medium; disposition: adopt;
roadmap_delta: TC009 bounded verified cache; verification_needed: current-policy
admission on hits, eviction, identity separation, fresh-state execution and timings.

Overall recommendation: select this measured cache slice under delegated scope
selection. WASI 0.3 and brokered I/O need separate ABI/capability designs and do not
address repeat compilation. Evidence gaps: synthetic timings are not a throughput
SLA; source-byte accounting cannot establish a compiled-code RSS bound.
