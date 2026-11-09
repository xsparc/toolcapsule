# DirectionBriefV1: Difference review

- schema_version: 1
- baseline_commit: 46044cd0a60890f0de8267cb99c965835986c075
- scanned_at: 2026-11-09
- coverage: saved handoff, live merge/base state, architecture, roadmap, relevant accepted decisions, readers, renderers, contracts and tests.

## Findings

1. CSP can restrict resource loads while a fixed style hash permits embedded CSS.
   [MDN CSP documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy).
   source_event_date: undated living documentation, accessed 2026-11-09;
   confidence: high; relevance: standalone review; direction_alignment: no scripts
   or external loads; opportunity_or_risk: hostile content and layout overflow;
   engineering_cost: low; disposition: adopt; roadmap_delta: TC024;
   verification_needed: escaped text, exact style hash and narrow-screen browser checks.
2. Strict model validation prevents ordinary coercion; explicit literal-version
   checks and semantic verification remain necessary.
   [Pydantic strict mode](https://docs.pydantic.dev/latest/concepts/strict_mode/).
   source_event_date: undated living documentation, accessed 2026-11-09;
   confidence: high; relevance: retained identity intake; direction_alignment:
   independent evidence checks; opportunity_or_risk: mistaking a matching digest
   for matching sources; engineering_cost: low; disposition: adopt;
   roadmap_delta: TC024; verification_needed: malformed records, byte counts,
   consistent replacement and source mismatch before output.

## Overall recommendation and evidence gaps

Complete the saved five-task scope with existing normalized contracts and bounded
review paths. Research informs implementation and does not authorize extra scope.
No signing, security audit, physical-device validation or measurement attestation
is claimed. Future CSV and source-checked rendering tasks remain planned.
