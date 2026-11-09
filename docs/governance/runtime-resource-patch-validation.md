# TC014 follow-up: Component resource accounting

Baseline: 4b7e27c. Reviewed 2026-11-09.

[RUSTSEC-2026-0424](https://rustsec.org/advisories/RUSTSEC-2026-0424.html)
covers host memory exhaustion through borrowed resources passed between
components. ToolCapsule executes signed source components and limits guest
linear memory, which does not account for the host allocations described in
the advisory. Upgrade to
[Wasmtime 49.0.5](https://github.com/bytecodealliance/wasmtime/releases/tag/v49.0.5)
within the existing WASI 0.2 profile. The other October 29 advisories concern
WASI 0.3 UDP, WASI HTTP, or embedder APIs for core Wasm reference types and
cross-engine struct types. ToolCapsule does not expose those paths.

Acceptance requires exact 49.0.5 pins and resolution, current-profile replay,
and rejection of receipts identifying 49.0.3 or 49.0.4. Existing receipts and
benchmark samples retain their historical profile.

Local validation passed on 2026-11-09. Formatting, all-target Clippy with
warnings denied, 64 Rust tests, three benchmark tests, four Python governance
tests and the static and dated strict evidence checks passed. The full Rust
suite used the existing WASI fixture. A separate guest build from the unchanged
source passed the benchmark correctness smoke on the new engine.

The lockfile updates 29 Wasmtime, Cranelift and Pulley packages. Cargo Audit
0.22.2 reported zero known vulnerabilities and zero warnings against RustSec
database revision 7a40350. Validation exercises the normal execution and
replay paths. It does not reproduce the advisory's exhaustion case.
