# DirectionBriefV1: Signed audit checkpoints

- schema_version: 1
- baseline_commit: 29e7b8a745ddf2235e070a40f6bd885e0b2a4d2b
- scanned_at: 2026-10-23
- coverage: merged milestone, roadmap, architecture, existing contracts and tests,
  open draft status, and primary guidance for this bounded extension.

Finding: The merged checkpoint milestone detects chain changes relative to a retained anchor, but custody remains entirely external. RFC 8032 specifies Ed25519, already used by the project for package signatures. Reusing that algorithm with a separate application domain and explicit host trust is a bounded extension. Source: [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032.html).

source_event_date: January 2017; confidence: high;
relevance: signed audit checkpoints; direction_alignment: portable verifiable evidence;
opportunity_or_risk: easier evidence review with explicit trust boundaries;
engineering_cost: low; disposition: adopt; roadmap_delta: TC012;
verification_needed: Round-trip authentication, tampering and wrong-domain rejection, revocation, strict schema and size admission, CLI exclusive output, required trust and prefix/exact verification.

Recommendation: implement this bounded follow-on to the merged milestone.
Broader ABI/broker changes for ToolCapsule and recovery or approval automation for
SimWeave remain separate candidate scopes; they require wider authority or state
transitions than this evidence extension. Evidence gaps: Signatures authenticate a retained statement, not freshness, completeness after its retained head, trusted time, or physical execution. An old valid signed checkpoint can be replayed; operators must independently retain their latest accepted anchor and enforce freshness. Signing a supplied checkpoint does not revalidate the source log. Key IDs select trusted public keys; aliases for the same key are equivalent. Provision checkpoint custodians separately from package publishers. No remote witness or key-management service is included.

