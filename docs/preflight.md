# Check package compatibility before execution

```sh
toolcapsule preflight --manifest signed.json --component guest.wasm --trust trust.json --policy policy.json
```

Preflight verifies current package authority, compiles source Wasm and checks the
WASI command interface. It emits version-1 JSON with the admitted manifest,
manifest SHA-256, signer, runtime profile, `compiled: true` and `executed: false`.
It reads no stdin and writes no receipt or audit record. Failure exits 2 with
sanitized diagnostics and no success report. `verify` retains its existing
compilation-free signature/digest/policy check.

Preflight proves compatibility with this engine configuration at the time of checking. It does not prove successful instantiation, execution, determinism, resource sufficiency or safety against engine defects. Compilation CPU/RSS are outside guest fuel/deadline limits and require an externally bounded worker. The report is unsigned and is not a reusable authorization token: reverify before later execution. No cache artifact, receipt, audit event or package download is produced.

Library callers can use `Runtime::preflight(VerifiedPackage)` after admission.
For continued execution, use `Runtime::prepare` and retain the returned tool;
preflight discards compiled code. No native serialized code is accepted.

