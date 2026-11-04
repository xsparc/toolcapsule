# Configuration templates

Status: accepted, 2026-10-27.

## Decision

Add `limits-template` for the existing validated default Limits contract and `policy-template [--limits limits.json]` for a host policy with an empty tool allowlist. Both print JSON by default or exclusively create `--out` files. Custom limits are bounded to 4096 bytes and validated before output. No keys, trust stores or guest execution are involved.

## Milestone items

1. Generate validated limits JSON for `pack --limits`.
2. Generate deny-by-default host-policy JSON, optionally using explicit limits.

## Acceptance

Generated limits round-trip through the existing contract. Generated policy denies a signed package until an operator explicitly adds its tool to the allowlist. Invalid and oversized limits produce no output; existing files are never overwritten.

## Limits

Templates are starting points, not authority grants or workload recommendations. Operators must choose appropriate budgets and allow tools explicitly. Existing signature, trust, capability and runtime enforcement remain in force. This milestone is stacked on security PR #18, including the signed-budget work merged there in #19.
