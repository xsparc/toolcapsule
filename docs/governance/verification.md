# Verification and validation

Acceptance is organized by the three roadmap slices. Package tests verify trust,
tampering, unsupported capability denial, schema rejection, and budget admission.
Runtime acceptance includes real Wasm execution and malicious fixtures. Adapter
acceptance includes CLI and MCP protocol exercises.

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test --locked`. Run `python scripts/check_governance.py`, then
`python scripts/check_governance.py --as-of 2026-10-17 --strict`.

TC001 validation on 2026-10-17: five package tests, formatting, Clippy, and the
project static and strict governance gates passed. Runtime, CLI, and cross-platform
validation are pending subsequent slices. No performance benchmark, independent
security review, hosted SaaS integration, or WASI 0.3 conformance is claimed.
