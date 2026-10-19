# TC008 verification

The shared pool is exercised with atomic reservations and a compiled local WASI
fixture. Tests cover cloned/independent pools, saturation, whole-window rejection,
input validation, empty batches, cancellation before task cleanup, capacity reuse,
denied capabilities and receipt replay. Existing batch tests cover ordered results,
fresh globals, panic/error cleanup and count/byte limits.

Local checks: 27 core/integration tests, three benchmark tests and four governance
regressions passed. Formatting and all-target Clippy passed. The cancellation tests
also passed after strengthening the assertion that capacity remains held immediately
after dropping the batch, before the executor cleans up its aborted tasks.

No throughput, RSS, tenant isolation or distributed-quota claim is made.
