# ADR 0002: Measure preparation and reuse

Status: accepted under delegated milestone selection, 2026-10-18.

TC005 adds a synthetic benchmark using existing verification/runtime interfaces.
Separate verification, preparation and invocation before tuning the engine.
Runtime behavior, capability policy and replay profile remain unchanged.

Acceptance: bounded arguments, correct nearest-rank summaries, rejection of failed
or inconsistent runs, real fixture execution, formatting, Clippy, tests and strict
governance. Expected paths: benchmark example, CI and documentation. Risk: noisy
local measurements mistaken for throughput guarantees. ABI expansion, external
capabilities and caches remain deferred. See docs/governance/benchmark-direction.md.
