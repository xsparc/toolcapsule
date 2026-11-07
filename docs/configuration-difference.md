# Configuration difference

Compare two strictly validated retained configuration summaries with explicit before-to-after direction. The result includes both summaries and a stable ordered list of changed JSON Pointer fields. Validity, each check status/error, deny-all flags and budget status/exceeded fields are compared. Missing optional fields count as changes. Identical summaries do not establish identical source files, grants or package admission.

```sh
toolcapsule configuration-diff before.json after.json --out difference.json
toolcapsule configuration-diff-schema
```

Each input is bounded to 4096 bytes; output is bounded to 16 KiB. Without `--out`, print JSON to stdout. Output files are created exclusively after both inputs validate; existing files are preserved. A valid difference returns success even when summaries describe invalid configuration. Malformed evidence fails with fixed diagnostics. The closed schema describes shape; the reader additionally enforces semantic consistency. This iteration exports differences; it does not add a retained-difference reader or source attestation.
