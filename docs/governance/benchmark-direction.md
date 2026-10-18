# DirectionBriefV1: execution measurements

- schema_version: 1
- baseline_commit: d42dbbd
- scanned_at: 2026-10-18
- coverage: focused preparation/reuse, ABI compatibility and measurement needs.

1. finding: Wasmtime separates compilation and instantiation and recommends
   pre-resolving imports for repeated execution.
   [Wasmtime guidance](https://docs.wasmtime.dev/examples-fast-instantiation.html).
   source_event_date: undated, accessed 2026-10-18; confidence: high;
   relevance: PreparedTool already uses CommandPre; direction_alignment: performance;
   opportunity_or_risk: measure before optimization; engineering_cost: low;
   disposition: adopt; roadmap_delta: TC005; verification_needed: real fixture,
   separate timings and deterministic result checks.
2. finding: WASI 0.3 adds native async; toolchain support varies.
   [WASI releases](https://wasi.dev/releases).
   source_event_date: undated, accessed 2026-10-18; confidence: high;
   relevance: future guest ABI; direction_alignment: portability;
   opportunity_or_risk: wider conformance surface; engineering_cost: medium;
   disposition: monitor; roadmap_delta: none; verification_needed: real 0.3 guests.

Overall recommendation: establish the missing measured baseline first, under the
2026-10-18 delegated scope-selection and draft-PR request. Evidence gaps: independent
sandbox review, parallel throughput and cross-platform performance. Reconsider ABI
expansion when a concrete guest needs it.
