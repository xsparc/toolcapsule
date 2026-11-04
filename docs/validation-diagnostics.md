# Machine-readable configuration diagnostics

Append `--json-errors` to `limits-check --limits limits.json`,
`policy-check --policy policy.json`, `trust-check --trust trust.json`, or
`budget-fit --limits limits.json --policy policy.json`.

A failed check emits only fixed fields on stderr, for example:

```json
{"schema_version":1,"kind":"limits","error":"invalid_format"}
```

Kinds are limits, policy, trust and budget_fit. Error codes are invalid_format
(malformed JSON/closed contract or invalid trust encoding), invalid_configuration
(invalid limits/policy semantics), input_too_large, io_error, validation_failed
(reserved fallback), and budget_exceeded (valid inputs whose budgets do not fit).
No input content, key IDs, key material or filesystem paths are included.

Exit 0 means the requested check passed, 1 means budget non-fit, and 2 means a
check could not complete. Budget non-fit still prints its normal JSON report on
stdout. Success stdout and default stderr are unchanged. CLI argument/help
errors are handled by Clap before these checks and retain their usual text.

Run `cargo test --locked --test configuration_workflow` after building the
existing WASI fixture as documented in CI. The test uses temporary synthetic
keys and files, creates no guest host grants and checks preflight without guest
execution. All validation commands are read-only. A valid trust file or fitting
budget does not authenticate or admit a package.
