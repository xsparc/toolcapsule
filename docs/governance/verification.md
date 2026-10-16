# Verification and validation

Acceptance is organized by the three roadmap slices. Package tests verify trust,
tampering, unsupported capability denial, schema rejection, and budget admission.
Runtime acceptance includes real Wasm execution and malicious fixtures. Adapter
acceptance includes CLI and MCP protocol exercises.

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test --locked`. Run `python scripts/check_governance.py`, then
`python scripts/check_governance.py --as-of 2026-10-17 --strict`.

TC001 validation on 2026-10-17: five package tests, formatting, Clippy, and the
project static and strict governance gates passed.

TC002 validation on 2026-10-17: all ten package/runtime tests passed, including
real WASI host-denial, blocked timer cancellation, fuel/memory exhaustion, fresh
state, and tampered replay evidence. Formatting, Clippy, and both governance
gates passed. Local execution was on Windows x64. CLI and cross-platform
validation are pending the final slice. No performance benchmark, independent
security review, hosted SaaS integration, or WASI 0.3 conformance is claimed.
