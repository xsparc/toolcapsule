# Explicit signed resource budgets

Status: accepted, 2026-10-26.

## Decision

Add optional `pack --limits limits.json` using the existing closed Limits contract. Bound input to 4096 bytes and require every field: positive fuel/input/output, memory at least 65536 bytes and timeout at least 10 milliseconds. Expose Limits::validate for budget shape validation without a host grant. Sign the chosen limits into the existing manifest; omitted --limits preserves defaults. Host policy still independently admits each requested budget at verify/preflight/run time. Reject malformed limits before reading the signing key or creating output. Keep exclusive output creation and the existing 16 MiB packaging cap.

## Acceptance

Custom limits appear unchanged in authenticated preflight output; input enforcement rejects oversized invocation; stricter host policy rejects the package; unknown/missing/zero/too-small/oversized limits fail before output; default packaging and non-overwrite behavior remain covered.

## Limits

A publisher requests budgets; it cannot grant host capabilities or increase the operator's policy. Fuel is engine-specific work accounting, not a portable wall-time estimate. Guest memory, fuel, I/O and timeout limits do not bound compilation CPU/RSS or all process allocations. No execution ABI or receipt profile changes are introduced by this slice. It is stacked on security patch PR #18 and must follow that patch.

