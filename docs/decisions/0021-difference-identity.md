# ADR 0021: Difference identity

Status: accepted
Date: 2026-11-11

## Decision


`configuration-diff-identity difference.json` prints a version-1 identity record
for a strictly validated difference. SHA-256 hashes the UTF-8 domain prefix
`toolcapsule:configuration-difference:v1` followed by LF, then compact JSON of
the validated envelope with recursively sorted object keys, preserved array
order, and no trailing newline. The encoding is `toolcapsule_difference_v1`.
The closed evidence contains fixed vocabulary, booleans and integral versions;
this is a project-specific encoding, not a claim of RFC 8785 JCS conformance.
A golden vector fixes the encoding; formatting and object-key order do not
change identity. Direction and changed summaries do.

```sh
toolcapsule configuration-diff-identity difference.json
toolcapsule configuration-diff-check difference.json --expected-sha256 DIGEST
toolcapsule configuration-diff-html difference.json --out review.html --expected-sha256 DIGEST --before before.json --after after.json
toolcapsule configuration-diff-csv difference.json --out review.csv --expected-sha256 DIGEST
toolcapsule configuration-diff-verify difference.json before.json after.json --expected-sha256 DIGEST
toolcapsule configuration-diff-csv-verify difference.json review.csv --expected-sha256 DIGEST
```

Replace DIGEST with the independently retained 64-character lowercase hex digest.
The optional gate validates syntax and identity before checking or exporting;
it never bypasses independent source checks. Existing calls remain compatible.
Difference intake remains 16 KiB, each source summary 4096 bytes; export limits
and exclusive output creation remain unchanged. Errors use fixed JSON categories
and exit 2. No guest execution, signing, trust-store mutation or file repair occurs.
Identity describes normalized lossy summaries, not original bytes, original source
configuration, authorship or execution authority. Retaining a replacement digest
with replacement evidence cannot detect substitution; retain the target independently.

## Alternatives and acceptance

Reuse existing semantic validators and renderers rather than parsing or repairing
rendered evidence. Preserve output compatibility and runtime authority boundaries.
Acceptance covers bounds, exact comparison, source/identity composition,
consistent replacement, fixed errors and input preservation. No layout changes.
