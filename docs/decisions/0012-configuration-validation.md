# Offline configuration validation

Status: accepted, 2026-10-27.

## Decision

Add read-only `limits-check --limits` and `policy-check --policy` commands. Limits input is capped at 4096 bytes and policy input at 64 KiB, with strict existing JSON parsing. Limits reuse Limits::validate. Policy::validate checks valid maximum budgets, a positive component cap, valid manifest tool names and unique allowlist entries; an empty allowlist remains valid. Return fixed version-1 JSON summaries without input values or paths. Validation does not load trust or signing keys, compile or execute code, or modify files.

## Milestone items

1. Validate limits files offline.
2. Validate policy files offline with explicit empty-allowlist reporting.

## Acceptance

Generated defaults pass both commands; invalid, unknown-field and oversized files fail with exit 2 and no success JSON. Policy checks reject duplicate/unusable names, zero component caps and invalid limits. Valid empty allowlists report denies_all_tools=true. Inputs stay unchanged and diagnostics do not echo supplied fields.

## Limits

Policy validation is an optional authoring check and does not change runtime admission. Deliberately unusable policies such as a zero component cap may still deny all work at runtime but fail this authoring check. Success is not a security audit, package verification or authority grant. ToolCapsule remains stacked on security PR #18.
