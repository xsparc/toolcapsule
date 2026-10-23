# TC011 verification

Synthetic chain tests cover exact and extended logs, internally valid prefix
rewrites and truncation, malformed suffixes, empty anchors, invalid schema/hash
forms and the existing log byte cap. Real CLI tests cover exclusive export,
anchored verification, exact-mode rejection, checkpoint byte limits and failure
before output creation.

Local verification passed: 35 core/integration Rust tests, three benchmark unit
tests, four governance regressions, cargo fmt --check, all-target Clippy with
warnings denied, and static/current-date strict governance checks. The existing
incremental-cache hard-link warning used Cargo's copy fallback; no test failed.
TC010 catalog tests belong to the separate open PR and are not included in this
main-based branch's test count.

No external checkpoint custody, signature service or remote witness is tested.
The checkpoint remains unsigned and must be independently protected.
