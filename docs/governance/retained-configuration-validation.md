# TC021 validation

Validated 2026-11-04 on merged main after the security stack landed.

- cargo fmt --check and all-target Clippy with warnings denied passed.
- Full Rust suite: 61 tests passed; three benchmark-example tests passed.
- Three new tests cover all eight valid/invalid check combinations, all error
  codes, contradictory validity/budget status, malformed deny-all shapes,
  duplicates, unknown fields, versions, field order, UTF-8 and size bounds.
- CLI acceptance verifies source equality, changed deny-all indicators, matching
  failure summaries, fixed diagnostics, unchanged evidence and no extra outputs.

Existing real WASI fixture reused; guest source is unchanged. Full checks ran
with normal local permissions; no new runtime authority or dependency was added.
The reader checks summary consistency; only explicit source verification can
check indicator truth, and even that does not establish identical source content.

Project-specific static and 2026-11-04 strict governance checks passed.
Four Python governance regressions passed.
The plugin self-checker was also attempted; it requires project identity
opensteward-control-plane and its own registry dialect, so it rejects these
independent project registries. The repository-specific gate is applicable here;
no project identity or historic registry was rewritten to satisfy that self-check.
