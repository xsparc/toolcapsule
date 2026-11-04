# TC019 validation

Validated 2026-10-29 against the TC018 implementation branch.

- cargo fmt --check: passed.
- cargo clippy --all-targets --locked -- -D warnings: passed.
- cargo test --locked: 56 tests passed, including two new configuration workflow tests.
- cargo test --locked --example benchmark: three tests passed.
- Python governance regression suite: four tests passed.
- Static and 2026-10-29 strict project governance checks: passed.

The synthetic workflow generates temporary templates and keys, checks limits,
policy and trust, signs the existing real WASI fixture, verifies and preflights
it, and proves allowlist and budget denials. JSON diagnostics cover malformed,
missing, oversized and invalid configuration with fixed vocabulary, unchanged
inputs, preserved default stderr and 0/1/2 exits.

The existing compiled guest fixture was reused; current guest source is
unchanged. CI rebuilds it and runs the acceptance test on all three platforms.
No new dependency audit, exploit test or guest capability was introduced.
The local filesystem required copy fallback for Rust incremental artifacts;
governance regression temporaries were placed in an ignored project directory.
These environment limitations did not require production code changes.
