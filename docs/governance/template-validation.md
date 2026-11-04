# TC016 validation

Generated limits round-trip through the existing contract. Generated policy denies a signed package until an operator explicitly adds its tool to the allowlist. Invalid and oversized limits produce no output; existing files are never overwritten.

Local validation passed: 48 core/integration Rust tests, three benchmark tests,
four governance regressions, cargo fmt --check, all-target Clippy with warnings
denied, and static/current-date strict governance checks. The two new CLI tests
cover template contracts, real signed-package policy denial/admission, invalid
inputs and exclusive creation. The existing compiled WASI fixture was reused.
Governance regressions passed with normal temporary-directory access after the
sandbox prevented temporary-directory creation; this was an environment failure.

The tested baseline includes Wasmtime 49.0.4 from PR #18. This does not claim a
new dependency audit, runtime exploit test or support for a new guest ABI.

Templates are starting points, not authority grants or workload recommendations. Operators must choose appropriate budgets and allow tools explicitly. Existing signature, trust, capability and runtime enforcement remain in force. This milestone is stacked on security PR #18, including the signed-budget work merged there in #19.
