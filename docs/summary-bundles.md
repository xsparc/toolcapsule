# Summary bundles

```sh
toolcapsule configuration-bundle-schema
toolcapsule configuration-bundle before.json after.json --out bundle.json
toolcapsule configuration-bundle-verify bundle.json --before before.json --after after.json
```

The closed version-1 envelope contains before/after summaries and their recomputed
difference. Typed readers reject duplicate and unknown fields, unsupported versions
and inconsistent nested evidence. Input summaries are capped at 4096 bytes, nested
normalized differences at 16 KiB and the whole bundle at 32 KiB. The schema describes
structure; the reader additionally enforces semantic relationships and byte bounds.

Export validates before exclusively creating its destination. Verification is
read-only and accepts either no source options or a complete before/after pair.
A partial pair, reversal or replacement mismatching independent summaries fails.
Self-consistent replacement remains valid standalone evidence. Failed configuration
checks remain valid descriptive summaries. Success exits 0; errors exit 2 with
fixed diagnostics. No guest executes. Summary consistency establishes neither
original configuration identity nor authority.
