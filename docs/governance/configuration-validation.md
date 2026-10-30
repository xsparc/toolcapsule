# TC017 validation

Generated defaults pass both commands; invalid, unknown-field and oversized files fail with exit 2 and no success JSON. Policy checks reject duplicate/unusable names, zero component caps and invalid limits. Valid empty allowlists report denies_all_tools=true. Inputs stay unchanged and diagnostics do not echo supplied fields.

Local validation passed: 50 core/integration Rust tests, three benchmark tests,
four governance regressions, cargo fmt --check, all-target Clippy with warnings
denied, and static/current-date strict governance. New tests cover read-only
CLI summaries, malformed/oversized input, no input echo, and semantic policy
errors. Existing signed-package and real WASI fixture tests still pass.

The existing compiled fixture was reused. No new sandbox exploit, dependency
audit, guest ABI, or external capability was tested.

Policy validation is an optional authoring check and does not change runtime admission. Deliberately unusable policies such as a zero component cap may still deny all work at runtime but fail this authoring check. Success is not a security audit, package verification or authority grant. ToolCapsule remains stacked on security PR #18.
