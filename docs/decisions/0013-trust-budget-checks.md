# Trust validation and budget-fit reporting

Status: accepted, 2026-10-28.

## Decision

Implement the two TC018 handoff tasks. `trust-check --trust` reads at most 64 KiB and validates a JSON object of 1–64-byte key IDs and hex-encoded 32-byte Ed25519 public keys. A custom Serde map visitor rejects duplicate decoded IDs; trailing data, malformed keys and weak points are rejected. Empty stores pass and report deny-all. `budget-fit --limits --policy` validates both inputs and reports fixed field names exceeding host maxima, sharing the runtime upper-bound comparisons. Inputs and trust files are never rewritten; summaries omit key material, IDs and paths.

## Milestone items

1. Validate trust-store IDs and public keys offline.
2. Report requested budget fit using shared admission comparisons.

## Acceptance

Empty and valid trust maps pass; duplicate/escaped-equivalent IDs, malformed types, invalid/weak/incorrect-length keys, trailing data and oversized inputs fail. Equality fits; each exceeded budget matches actual package denial. CLI differentiates fit, non-fit and invalid input, reports no raw input and leaves files unchanged.

## Limits

Trust validation proves encoding and non-weakness under the pinned dalek implementation, not ownership, identity or authority. It is an opt-in authoring check; runtime trust parsing is unchanged. Budget fit checks only limits, not signer, capabilities, component size, tool allowlist or package authenticity. The empty tool allowlist can therefore deny a package whose budget fits. Security PR #18 is still the required patched baseline.
