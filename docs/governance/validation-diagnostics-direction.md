# DirectionBriefV1: Validation diagnostics and configuration acceptance

- schema_version: 1
- baseline_commit: 4cbd77ed073840eb18841a7d1ed4a6e7db8921bd
- scanned_at: 2026-10-29
- coverage: focused milestone handoff, repository architecture/roadmap/ADRs, current CLI/contracts/tests, dependency PR state and primary documentation relevant to offline review.

Finding: Clap supports command-local opt-in flags. Fixed validation diagnostics can be added without changing successful output, parser syntax or the runtime boundary.

Source: [Primary documentation](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html).
source_event_date: undated living documentation, accessed 2026-10-29.
confidence: high; relevance: validation diagnostics and configuration acceptance;
direction_alignment: existing bounded local workflows;
opportunity_or_risk: make invalid or inconsistent retained inputs explicit;
engineering_cost: low to medium; disposition: adopt; roadmap_delta: TC019;
verification_needed: Fixed JSON codes and unchanged 0/1/2 exits; no input echo; synthetic signed fixture preflight with explicit allowlist and budget denials.

Overall recommendation: Add --json-errors to the four existing checks and a temporary synthetic configuration workflow tested by the current CI matrix.
The user requested implementation of the retained tasks or selection of the first
five when absent, with draft PRs and a subsequent backlog. That request supplies
scope authority; external sources are supporting evidence only.

Evidence gaps: no independent security audit or physical-device validation.
The focused scan does not establish absence of dependency vulnerabilities.
No runtime, simulator, broker, hosting or release boundary changes are needed.
Future backlog items remain planned until the next authorized iteration.
