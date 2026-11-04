# TC020 validation

Validated 2026-10-31 on the merged TC019 security-branch base.

- cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings: passed.
- cargo test --locked: 58 tests passed; benchmark example: three tests passed.
- Combined-check tests cover independent failures, invalid-input precedence,
  fit/non-fit, deny-all, unchanged files and explicit required arguments.
- Schema tests compare the embedded/exported artifact, closed required fields,
  real CLI success discriminators and diagnostic vocabulary. These are contract
  drift tests, not an independent JSON Schema implementation or semantic verifier.

Existing real WASI fixtures were reused; guest source is unchanged. Windows
blocked a test executable in the sandbox; the normal-access rerun passed.
Incremental compilation used a copy fallback where hardlink creation failed.
No runtime authority, new production dependency or security-audit claim is added.

Static and 2026-10-31 strict project governance checks passed.
Four Python governance regression tests passed; the expanded CLI schema
contract test passed after checking every success discriminator and field set.
