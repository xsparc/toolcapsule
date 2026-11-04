# Validation diagnostics and configuration acceptance

Status: accepted, 2026-10-29. Scope: the two TC019 tasks in the retained five-item handoff.

Add `--json-errors` to limits-check, policy-check, trust-check and budget-fit.
Errors are one version-1 JSON object on stderr with fixed kind/error vocabulary.
Success stdout and exit codes remain compatible: 0 valid/fit, 1 non-fitting
budgets, 2 invalid input or I/O. Budget non-fit retains its stdout report and
adds budget_exceeded on stderr only when requested. Other codes are invalid_format,
invalid_configuration, input_too_large, io_error and validation_failed.
No supplied values, IDs, keys or paths enter these diagnostics. Clap argument
syntax/help errors retain their existing interface; this flag covers checks after
argument parsing. Templates, execution and MCP output contracts are unchanged.

A synthetic integration fixture generates templates and disposable keys, checks
all three configuration types, signs the existing WASI fixture, demonstrates
allowlist denial, explicitly permits that fixture, verifies and preflights it,
then demonstrates budget non-fit and package denial. Preflight executes no guest.
The existing cargo test CI matrix discovers this acceptance test on every OS.

Acceptance: fixed codes for malformed, oversized, missing and semantically invalid
files; unchanged inputs; no private markers in diagnostics; exact default stderr;
0/1/2 behavior; signed preflight success and both explicit denials.

No guest capabilities, trust authority, runtime dependencies or execution profile
change. This follows the current handoff rather than adding a broker or hosted
service. See the validation record and configuration diagnostics guide.
