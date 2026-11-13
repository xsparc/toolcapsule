# ADR 0022: Retained identity html

Status: accepted
Date: 2026-11-12

## Decision


```sh
toolcapsule configuration-diff-identity-schema
toolcapsule configuration-diff-check difference.json --identity identity.json
toolcapsule configuration-diff-html-verify difference.json review.html --before before.json --after after.json --identity identity.json
```

The existing version-1 identity record now has a closed JSON Schema and strict
reader capped at 4096 bytes. All fields are required; unknown/duplicate fields,
unsupported encoding/version, non-integer version and malformed lowercase
64-character SHA-256 digests fail. The identity generation encoding is unchanged.
All existing digest-gated check, source verification, HTML/CSV export and CSV
verification commands accept --identity, mutually exclusive with --expected-sha256.
Identity checks never replace independent source checks.

HTML verification reads at most 64 KiB, validates the retained difference (16 KiB)
and optional paired summaries (4096 bytes each), then compares exact renderer
bytes. **Use the same source-check mode as export:** the existing HTML status text
differs between standalone and explicit-source rendering. Optional identity has
no effect on the rendered bytes. Markup, CSP, style, encoding and newline edits
fail without repair or writes. Success prints fixed JSON and exits 0; invalid
input/mismatch exits 2. Existing renderer layout and command defaults are unchanged.

No guest execution or browser is involved. Identity records describe normalized
lossy summaries, not signatures, source configuration identity or execution authority.
A self-consistent replacement with a replacement identity requires independent
summaries or an independently retained identity target to detect substitution.

## Alternatives and acceptance

Reuse existing bounded semantic readers and renderers rather than accepting unvalidated JSON or repairing exports. Acceptance covers closed contracts, strict versions, component/whole limits, independent source gates, replacement evidence, exact HTML/CSV bytes and read-only verification. No runtime authority or layout changes.
