# TC018 validation

Empty and valid trust maps pass; duplicate/escaped-equivalent IDs, malformed types, invalid/weak/incorrect-length keys, trailing data and oversized inputs fail. Equality fits; each exceeded budget matches actual package denial. CLI differentiates fit, non-fit and invalid input, reports no raw input and leaves files unchanged.

Local validation passed: 54 core/integration Rust tests, three benchmark tests,
four governance regressions, cargo fmt --check, all-target Clippy with warnings
denied, and static/current-date strict governance. New regressions cover decoded
duplicate IDs, invalid/weak keys, all five budget boundaries against signed
package admission, CLI exit codes, read-only behavior and sanitized output.
The existing compiled WASI fixture was reused. No new runtime exploit or
dependency audit is claimed.

Trust validation proves encoding and non-weakness under the pinned dalek implementation, not ownership, identity or authority. It is an opt-in authoring check; runtime trust parsing is unchanged. Budget fit checks only limits, not signer, capabilities, component size, tool allowlist or package authenticity. The empty tool allowlist can therefore deny a package whose budget fits. Security PR #18 is still the required patched baseline.
