# DirectionBriefV1: retained audit anchors

- schema_version: 1
- baseline_commit: 5cedfd561ddf8fc243db8dd7989ca38da9e763c0
- scanned_at: 2026-10-22
- coverage: audit writer/verifier, security guidance, open TC010 status and primary
  append-only log consistency literature. Focused on the existing evidence boundary.

Finding: ToolCapsule already requires externally retained chain heads in its threat
model but has no command to compare a current log with that retained evidence.
[RFC 9162](https://www.rfc-editor.org/rfc/rfc9162.html#section-2.1.4) distinguishes
log consistency from a single current snapshot. Applying that principle to this
existing linear hash chain suggests checking a trusted prefix count/hash.
source_event_date: December 2021; accessed 2026-10-22; confidence: high;
relevance: detect rewrites and truncation before a retained head;
direction_alignment: digest-only auditable execution; opportunity_or_risk: useful
retention checks versus false trust in colocated unsigned checkpoints;
engineering_cost: low; disposition: adopt; roadmap_delta: TC011 checkpoint export
and anchored verification; verification_needed: valid rewrites, truncation, suffix
validation, strict checkpoint parsing and exclusive creation.

Overall recommendation: close the existing local verification gap independently
of the open catalog draft. This is a project-specific linear-chain check, not a
Certificate Transparency implementation or Merkle consistency proof. Evidence gaps:
checkpoint custody is operator-owned; signatures, remote witnesses and physical
execution attestation are not provided. Brokered I/O and ABI changes remain separate.
