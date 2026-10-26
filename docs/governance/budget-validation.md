# TC015 validation

Custom limits appear unchanged in authenticated preflight output; input enforcement rejects oversized invocation; stricter host policy rejects the package; unknown/missing/zero/too-small/oversized limits fail before output; default packaging and non-overwrite behavior remain covered.

Local validation passed: 46 core/integration Rust tests, three benchmark tests,
four governance regressions, cargo fmt --check, all-target Clippy with warnings
denied, and static/current-date strict governance checks. Two new CLI regressions
exercise authenticated custom limits, real guest input enforcement, stricter host
policy rejection, invalid/oversized inputs and exclusive output creation.
The existing compiled WASI fixture was reused; its source was unchanged.

The tested baseline is the patched Wasmtime 49.0.4 branch from PR #18. This slice
does not claim a fresh complete dependency audit or a new sandbox exploit test.

A publisher requests budgets; it cannot grant host capabilities or increase the operator's policy. Fuel is engine-specific work accounting, not a portable wall-time estimate. Guest memory, fuel, I/O and timeout limits do not bound compilation CPU/RSS or all process allocations. No execution ABI or receipt profile changes are introduced by this slice. It is stacked on security patch PR #18 and must follow that patch.

