# TC012 verification

Round-trip authentication, tampering and wrong-domain rejection, revocation, strict schema and size admission, CLI exclusive output, required trust and prefix/exact verification.

Local validation passed: 39 core/integration Rust tests, three benchmark tests,
four governance regressions, cargo check, cargo fmt --check, and static/current-date
strict governance checks. All signing fixtures use synthetic keys.

Local all-target Clippy was attempted but Windows Application Control blocked the
cargo-clippy executable (OS error 4551). The existing three-platform PR workflow
must supply this remaining lint evidence; no local lint pass is claimed.

Limits: Signatures authenticate a retained statement, not freshness, completeness after its retained head, trusted time, or physical execution. An old valid signed checkpoint can be replayed; operators must independently retain their latest accepted anchor and enforce freshness. Signing a supplied checkpoint does not revalidate the source log. Key IDs select trusted public keys; aliases for the same key are equivalent. Provision checkpoint custodians separately from package publishers. No remote witness or key-management service is included.

