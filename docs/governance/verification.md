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
validation are covered by the final slice below. No performance benchmark, independent
security review, hosted SaaS integration, or WASI 0.3 conformance is claimed.

TC003 local validation on 2026-10-17: all 15 tests passed. Real CLI subprocesses
cover key generation without overwrite, signing, digest rejection, execution,
replay, audit verification, corrupted log rejection, and MCP lifecycle/errors.
Formatting and Clippy with warnings denied passed. The documented demo printed
the expected uppercase text and matched its replay. Cargo audit checked the
locked dependencies against RustSec with no reported advisories. Static and
strict governance checks passed. Hosted Windows/Linux/macOS CI is configured;
its current results are attached to PR #3 rather than inferred from local tests.
