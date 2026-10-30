# TC014: runtime security patch

Baseline: 82f8997, reviewed 2026-10-26. Scope is the 49.0.3 to 49.0.4 engine
patch and corresponding replay-profile identity, within the existing WASI 0.2
design. The upstream release fixes
[GHSA-mghf-wjfp-p5xm](https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-mghf-wjfp-p5xm),
a critical Cranelift AArch64 branch-table miscompilation. RustSec lists 49.0.4
as patched in [RUSTSEC-2026-0379](https://rustsec.org/advisories/RUSTSEC-2026-0379.html).
The separate [stream-lowering advisory](https://rustsec.org/advisories/RUSTSEC-2026-0378.html)
also lists 49.0.4 as patched; this project does not enable the WASI 0.3 stream ABI.

Acceptance: exact dependency pins and lockfile resolve to 49.0.4; the new profile
round-trips current receipts and rejects the legacy 49.0.3 profile; existing
isolation, admission, cache, checkpoint, preflight and governance checks pass.
The migration regression failed on the baseline because it retained the old
profile. Historical benchmark/audit evidence must not be rewritten.

Local validation on 2026-10-26 passed: all 44 core/integration Rust tests,
three benchmark tests, four governance tests, formatting, all-target Clippy
with warnings denied, and static/current-date strict governance checks.
Both direct engine pins resolve to 49.0.4; all 29 lockfile changes belong to
Wasmtime, Cranelift or Pulley, with no unrelated dependency updates.

Local tests used the existing 2,840,199-byte guest fixture. Its source is
unchanged; a fresh build was attempted but host application control blocked
the WASI linker. Hosted CI rebuilds the fixture from source and records its
platform results on the draft PR.

No exploit payload or sandbox-escape reproduction is run.
A full fresh local dependency audit could not execute because host application
control blocked the audit executable. Upstream advisory ranges were reviewed
directly; this is not a clean full-lockfile audit result. Existing standard CI
covers Windows, Linux and macOS, whose public standard macos-latest runner is
AArch64 according to [GitHub runner documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).
No release, deployment, evidence-format change or guest capability expansion is
part of this patch.
