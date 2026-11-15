# ADR 0025: Bundle review

Status: accepted
Date: 2026-11-15

## Decision

```sh
toolcapsule configuration-bundle-render bundle.json --format html --out review.html --before before.json --after after.json
toolcapsule configuration-bundle-verify-export bundle.json review.html --format html --before before.json --after after.json
```

Rendering defaults to HTML; verification requires --format html or csv. Both accept
optional paired independent summaries. The SDK render and verify_export functions
reuse strict bundle intake and the existing difference renderers. Validation and
source comparison precede exclusive output creation. Verification never writes.
Bundles remain bounded at 32 KiB, summaries at 4096 bytes and HTML/CSV at 64 KiB.
Unknown formats, partial pairs, reversal, inconsistent bundles and edited exports
fail with exit 2 and fixed diagnostics. Failed configuration evidence remains valid.

HTML explicitly states whether independent summaries were supplied. Exact HTML
verification must use the same source-check mode as rendering. CSV preserves its
existing byte format; supplying sources still checks them before rendering or
verification. Invalid hostile summary strings fail intake; accepted values reuse
existing HTML escaping and CSV literal-cell handling. No guest executes. Internal
consistency cannot detect a consistent replacement without independent sources.

## Alternatives and acceptance

Reuse strict readers and existing renderers/identity encodings instead of introducing parallel interpretations. Acceptance: `tests/bundle_review.rs`. Bounds, negative inputs, independent checks and byte preservation are covered. Runtime execution and authority changes are outside this slice.
