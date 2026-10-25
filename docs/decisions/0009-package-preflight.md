# Package preflight

Status: accepted, 2026-10-25.

## Decision

Add `preflight` using the same package arguments as `verify`. Verify current trust, digest and policy first, then compile source bytes and validate imports and the WASI command interface through the existing Runtime::prepare path. Return a version-1 JSON report containing the admitted manifest, exact manifest digest, signer, runtime profile, compiled=true and executed=false. The library exposes Runtime::preflight. No store or instance is created, including no core start execution. Failed admission or preparation produces no success JSON and exits 2. Existing verify remains a compilation-free admission check.

## Acceptance

Real WASI fixture identity and profile; malformed source, absent command export and unresolved imports fail; valid signed invalid source distinguishes verify from preflight; trust/digest denial precedes compilation; trapping start and command code are not run.

## Limits

Preflight proves compatibility with this engine configuration at the time of checking. It does not prove successful instantiation, execution, determinism, resource sufficiency or safety against engine defects. Compilation CPU/RSS are outside guest fuel/deadline limits and require an externally bounded worker. The report is unsigned and is not a reusable authorization token: reverify before later execution. No cache artifact, receipt, audit event or package download is produced.

