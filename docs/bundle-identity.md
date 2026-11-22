# Bundle identity

```sh
toolcapsule configuration-bundle-identity-schema
toolcapsule configuration-bundle-identity bundle.json --out identity.json
toolcapsule configuration-bundle-verify bundle.json --identity identity.json --before before.json --after after.json
toolcapsule configuration-bundle-render bundle.json --out review.html --identity identity.json
toolcapsule configuration-bundle-verify-export bundle.json review.html --format html --identity identity.json
cargo run --locked --example bundle_review -- target/debug/toolcapsule review-demo
```

The normalized identity validates the closed summary bundle, serializes compact
serde_json with recursively sorted object keys and original array order, then
hashes the domain prefix `toolcapsule:configuration-bundle:v1` plus LF followed by
those JSON bytes (no trailing LF). Encoding is toolcapsule_bundle_v1, not JCS.
The bytes field counts the normalized JSON only, excluding the domain prefix.
The synthetic golden fixture pins 1624 bytes and digest
bc159345e23e65a16d370806e0c25c047736645b4b9e276d61a61a4317764b49.

The closed identity reader rejects duplicates, missing/unknown fields, invalid
versions, uppercase or malformed digests and non-integer/out-of-range byte counts.
Record input is limited to 4096 bytes and normalized bundle bytes to 32768.
--out exports pretty UTF-8 JSON plus LF exclusively; without it identity goes to
stdout. Bundle verification, rendering and exact export verification accept either
--expected-sha256 or --identity, never both. Identity files check digest and byte
count before any output creation and compose with independent summaries.

The new example requires a new destination directory and uses only allowlisted
non-executing review commands. It tests both formats, output collisions, consistent
replacement and byte preservation. On Windows use target/debug/toolcapsule.exe.
HTML verification retains the same explicit-source mode as rendering. Identity
proves normalized content consistency, not original source identity or signatures.

## Read the verification scope

Successful `configuration-bundle-verify` and
`configuration-bundle-verify-export` responses include two additive booleans:

| Field | True only when |
| --- | --- |
| `source_checked` | Both supplied summaries were checked against the bundle. |
| `identity_checked` | The bundle matched an explicit expected digest or an identity file. |

Both are false in standalone mode. The export command also checks the retained
export bytes against the rendered result. A failed check emits no success JSON.
The flags record what was compared, not who controlled the comparison inputs.
Retain the summaries or identity target independently if substitution is a
concern. Neither flag establishes authorship, signer trust, or execution
authority. Consumers handling older responses should treat a missing flag as
unknown, not as a verified check.
