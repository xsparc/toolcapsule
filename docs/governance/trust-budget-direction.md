# DirectionBriefV1: Trust validation and budget-fit reporting

- schema_version: 1
- baseline_commit: 789137a2678788b9328bfd853dcf7323cfcac1df
- scanned_at: 2026-10-28
- coverage: repository roadmap, architecture, merged and open PRs, current contracts and tests, primary security guidance.

Finding: The recorded handoff extends authoring checks to trust input and budget compatibility. Pinned dalek documents from_bytes point decoding and is_weak separately; both checks are needed for this authoring validator, without claiming RFC/NIST identity certification.

Source: [OWASP guidance](https://docs.rs/ed25519-dalek/2.2.0/ed25519_dalek/struct.VerifyingKey.html).
source_event_date: undated living documentation, accessed 2026-10-28;
confidence: high; relevance: trust validation and budget-fit reporting; direction_alignment: explicit bounded workflows;
opportunity_or_risk: reject ambiguous trust files and report budget incompatibility before package operations;
engineering_cost: low; disposition: adopt; roadmap_delta: TC018;
verification_needed: Empty and valid trust maps pass; duplicate/escaped-equivalent IDs, malformed types, invalid/weak/incorrect-length keys, trailing data and oversized inputs fail. Equality fits; each exceeded budget matches actual package denial. CLI differentiates fit, non-fit and invalid input, reports no raw input and leaves files unchanged.

Duplicate detection uses a custom map visitor before insertion can discard a
repeated ID, following Serde's [map deserialization API](https://serde.rs/deserialize-map.html).
Source event date: undated documentation, accessed 2026-10-28. Disposition: adopt.

Overall recommendation: implement the recorded TC018 handoff and keep runtime
trust semantics unchanged. A new ABI or broker boundary needs independent design
and isolation evidence and is not needed for these authoring checks.

Evidence gaps: Trust validation proves encoding and non-weakness under the pinned dalek implementation, not ownership, identity or authority. It is an opt-in authoring check; runtime trust parsing is unchanged. Budget fit checks only limits, not signer, capabilities, component size, tool allowlist or package authenticity. The empty tool allowlist can therefore deny a package whose budget fits. Security PR #18 is still the required patched baseline.
