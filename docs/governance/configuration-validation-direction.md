# DirectionBriefV1: Offline configuration validation

- schema_version: 1
- baseline_commit: 11d01353e5d5dbd2f03d559336d30d042b8314e9
- scanned_at: 2026-10-27
- coverage: repository roadmap, architecture, merged and open PRs, current contracts and tests, primary security guidance.

Finding: Merged templates make configuration easy to create, but edited files can only be checked as part of a package operation. Early syntactic and semantic validation follows OWASP guidance and lets authors find mistakes without signing or loading guest code.

Source: [OWASP guidance](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html).
source_event_date: undated living documentation, accessed 2026-10-27;
confidence: high; relevance: offline configuration validation; direction_alignment: explicit bounded workflows;
opportunity_or_risk: catch edited configuration mistakes before package operations without exposing configuration values;
engineering_cost: low; disposition: adopt; roadmap_delta: TC017;
verification_needed: Generated defaults pass both commands; invalid, unknown-field and oversized files fail with exit 2 and no success JSON. Policy checks reject duplicate/unusable names, zero component caps and invalid limits. Valid empty allowlists report denies_all_tools=true. Inputs stay unchanged and diagnostics do not echo supplied fields.

Overall recommendation: finish the existing configuration authoring workflow
before extending the guest ABI or broker capabilities. The previous five-item
iteration is complete in PRs #20 and xsparc/simweave#28; no unimplemented handoff
was recorded, so this slice initializes the next bounded work.

Evidence gaps: Policy validation is an optional authoring check and does not change runtime admission. Deliberately unusable policies such as a zero component cap may still deny all work at runtime but fail this authoring check. Success is not a security audit, package verification or authority grant. ToolCapsule remains stacked on security PR #18.
