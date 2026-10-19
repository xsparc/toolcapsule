# TC007 validation

2026-10-19: scheduler tests cover active-window bounds, reordered completion,
drop cancellation without starting queued work, infrastructure-error cleanup and
sanitized task panics. Runtime tests cover 64 fresh-state invocations, bounded
admission and arithmetic overflow, mixed real WASI timeout/success/error/denial
outcomes, unchanged receipt replay and cancellation of blocked guests followed by
successful reuse. No ambient capabilities or native artifact loading were added.

Final verification passed: 24 library/integration tests, three benchmark tests,
four governance CLI tests, formatting, Clippy with warnings denied and static/dated
strict governance. The batch example built and emitted eight successful ordered
receipts from a newly signed local synthetic fixture. Initial sandbox incremental
cache hard-link warnings were avoided with the final host-local checks.

Limits: cooperative cancellation requires a live executor, deadlines
exclude queued wait, limits are per batch and do not bound total worker memory or
other callers. No throughput speedup or physical multi-host claim.
