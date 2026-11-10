# DirectionBriefV1: Difference csv

- schema_version: 1
- baseline_commit: 6f5d36ed5c168643a14f4bb31e808837374e9d01
- scanned_at: 2026-11-10
- coverage: saved handoff, live merge/base state, architecture, roadmap, accepted decisions, exporters/verifiers, tests and focused CSV documentation.

## Findings

CSV dialects vary. [RFC 4180](https://www.rfc-editor.org/rfc/rfc4180), published
October 2005, documents CRLF records and quote escaping; it is informational.
[Python CSV documentation](https://docs.python.org/3/library/csv.html), undated
living documentation accessed 2026-11-10, describes explicit dialect/newline
handling. Choose exact bytes for retained-export verification, preserving existing
Python exporters and defining a fixed ToolCapsule profile.

source_event_date: as above; confidence: high; relevance: portable review tables;
direction_alignment: explicit evidence contracts; opportunity_or_risk: silent
normalization hides edits and spreadsheet text can be misinterpreted;
engineering_cost: low; disposition: adopt; roadmap_delta: TC025;
verification_needed: quote escaping, CRLF, ordered rows, literal text prefixes,
source mismatch despite matching identity, output bounds and no overwrite.

## Overall recommendation and evidence gaps

Complete the five saved tasks through existing source and identity validators.
Research informs design, not additional authority. No spreadsheet application
compatibility guarantee, security audit, signature, physical-device validation
or performance equivalence is claimed. Next tasks remain planned.
