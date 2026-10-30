# Check explicit configuration

```console
toolcapsule validation-schema
toolcapsule configuration-check --limits limits.json --policy policy.json --trust trust.json
```

The schema is also available as `schemas/validation.schema.json`. It describes
the four single-check success summaries, fixed `--json-errors` envelopes and the
combined result. Examples cover invalid_format, invalid_configuration,
input_too_large, io_error, validation_failed (reserved fallback) and
budget_exceeded. Parser usage errors are outside this JSON contract.

The combined check reads limits up to 4 KiB and policy/trust up to 64 KiB each.
Every input is checked even when another fails. `checks` reports valid or invalid
with a fixed error code; valid policy/trust include explicit deny-all indicators.
`valid` means all three inputs validated. `budget_fit.status` is fits, exceeds or
not_checked, with fixed exceeded field names. It can report exceeds even if the
independent trust check failed. Invalid inputs exit 2, otherwise exceeds exits 1,
and fits exits 0. The complete result is on stdout; invalid input values and paths
are never echoed. Input files are never rewritten.

`package_admission` is always not_checked. Exit 0 may accompany both deny-all
indicators: it means well-formed configuration and matching budgets, not authority
to run anything. These summaries cannot prove identical underlying policies or
trust stores and do not attest to who produced them. Retain original inputs
separately where needed, and use verify/preflight for actual package checks.
