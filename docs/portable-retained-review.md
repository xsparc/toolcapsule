# Portable retained review

```sh
toolcapsule configuration-diff-identity difference.json --out identity.json
cargo build --locked
cargo run --locked --example retained_review -- target/debug/toolcapsule review-demo
```

On Windows the executable argument is `target/debug/toolcapsule.exe`. The example
requires an existing parent and a new destination directory. It invokes only a
fixed allowlist of non-executing configuration-review CLI commands. It creates
synthetic summaries and their difference, exports an identity, renders HTML/CSV,
verifies both against independent summaries and identity, tests output collisions,
and rejects consistent replacement evidence even with a matching replacement
identity. Original summaries, difference, identity and exports remain unchanged.
No guest component, signing key or runtime execution is involved.

Identity generation without --out preserves stdout JSON. With --out it serializes
the same version-1 record as pretty UTF-8 JSON plus LF, validates the closed record
and 4096-byte bound, then creates the file exclusively. Successful file export has
no stdout. Invalid differences (16 KiB limit) and existing destinations fail with
exit 2 and fixed diagnostics before overwriting anything.

The example prints a fixed verified/guest_execution summary; errors do not echo
paths or command output. HTML export and verification use the same explicit-source
mode because the renderer includes source-check status text. Summary identity is
not a signature or original-configuration identity. The fixture is synthetic,
including the deliberately non-fitting budget in the after-summary.
