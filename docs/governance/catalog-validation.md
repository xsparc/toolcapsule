# TC010 verification

Catalog tests exercise real CLI signing, admission, stdio negotiation, lexical
pagination, named calls and shared audit identities. Rejection checks cover empty,
oversized and duplicate catalogs, unknown fields, invalid versions, missing files,
revoked trust, denied policy, changed component bytes and aggregate source limits.
An intentionally non-Wasm signed fixture proves byte admission precedes compilation.

Local verification passed: 34 core/integration Rust tests, three benchmark unit
tests, four governance regressions, cargo fmt --check, all-target Clippy with
warnings denied, and static/current-date strict governance checks. The existing
single-tool CLI, runtime isolation and receipt replay regressions passed.

The first governance regression run encountered sandbox temporary-directory ACL
errors; rerunning with normal temporary-directory access passed. Cargo used a
copy fallback for an existing incremental-cache hard-link warning. External MCP
client UIs, new protocol versions and multi-tenant deployments were not tested.
