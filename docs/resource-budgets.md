# Author signed resource budgets

Create a complete limits JSON file, for example:

```json
{"fuel":1000000,"memory_bytes":67108864,"timeout_ms":500,"input_bytes":1024,"output_bytes":4096}
```

```sh
toolcapsule pack --component guest.wasm --name example --key signing.key --limits limits.json --out signed.json
toolcapsule preflight --component guest.wasm --manifest signed.json --trust trust.json --policy policy.json
```

All five fields are required and unknown fields are rejected. The file must be
at most 4096 bytes. Fuel/input/output must be positive, memory at least 65536 bytes
and timeout at least 10 milliseconds. Omitting --limits keeps the existing defaults.
Existing outputs are never overwritten. The chosen values are authenticated as
part of the manifest; editing them requires a new signature.

A publisher requests budgets; it cannot grant host capabilities or increase the operator's policy. Fuel is engine-specific work accounting, not a portable wall-time estimate. Guest memory, fuel, I/O and timeout limits do not bound compilation CPU/RSS or all process allocations. No execution ABI or receipt profile changes are introduced by this slice. It is stacked on security patch PR #18 and must follow that patch.

Library callers may use Limits::validate to check budget shape. This method is
not host admission; VerifiedPackage::verify still checks operator policy.

