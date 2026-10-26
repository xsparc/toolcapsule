# DirectionBriefV1: Configuration templates

- schema_version: 1
- baseline_commit: 813f323327bff8fd6d71e6e75af750e3ba8395ff
- scanned_at: 2026-10-27
- coverage: repository roadmap, architecture, merged and open PRs, current contracts and tests, primary security guidance.

Finding: Existing budget and policy contracts are manually authored. A generator makes them discoverable while preserving OWASP's deny-by-default authorization principle.

Source: [OWASP guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html).
source_event_date: undated living documentation, accessed 2026-10-27;
confidence: high; relevance: configuration templates; direction_alignment: explicit bounded workflows;
opportunity_or_risk: prevent malformed hand-authored configuration while preserving explicit tool grants;
engineering_cost: low; disposition: adopt; roadmap_delta: TC016;
verification_needed: Generated limits round-trip through the existing contract. Generated policy denies a signed package until an operator explicitly adds its tool to the allowlist. Invalid and oversized limits produce no output; existing files are never overwritten.

Overall recommendation: implement template generation before adding a new guest ABI
or broker capability. Both alternatives expand the runtime boundary and need
separate interoperability and isolation evidence; this slice completes the
existing signed-budget authoring workflow.

Evidence gaps: Templates are starting points, not authority grants or workload recommendations. Operators must choose appropriate budgets and allow tools explicitly. Existing signature, trust, capability and runtime enforcement remain in force. This milestone is stacked on security PR #18, including the signed-budget work merged there in #19.
