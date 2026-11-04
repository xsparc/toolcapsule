# Verify retained configuration summaries

```console
toolcapsule configuration-check --limits limits.json --policy policy.json --trust trust.json > result.json
toolcapsule configuration-result-check --report result.json
toolcapsule configuration-result-verify --report result.json --limits limits.json --policy policy.json --trust trust.json
```

Capture stdout as UTF-8. The reader accepts at most 4 KiB and only the combined
configuration result. It rejects unknown/duplicate fields, unsupported or
noninteger versions, invalid deny-all shapes, contradictory validity, unavailable
budget checks, empty exceeds lists and repeated/out-of-order fields. Whitespace
and object property order do not affect semantic comparison.

The checker returns kind=configuration_result, valid=true and
local_consistency_only. The verifier returns verified=true and
local_source_consistency_only. Both exit 0 for valid evidence, even if it reports
invalid configuration or non-fitting budgets. Errors exit 2 with a fixed JSON
diagnostic; argument syntax errors remain CLI usage errors. validation-schema
includes the new summaries and diagnostic kind.

All paths are explicit, reads bounded and files unchanged. Source limits retain
the existing configuration-check caps. Reads are not an atomic snapshot. Missing
sources can reproduce matching failure summaries; verified does not assert
availability. Deny-all indicators are checked against sources only by the verifier.
Different trust stores or policies can produce identical summaries. Equality
does not identify keys, authenticate a producer, grant capability or admit a
package. Use package verify/preflight for actual package checks.
