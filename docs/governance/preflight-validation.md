# TC013 validation

Real WASI fixture identity and profile; malformed source, absent command export and unresolved imports fail; valid signed invalid source distinguishes verify from preflight; trust/digest denial precedes compilation; trapping start and command code are not run.

Local validation passed: 43 core/integration Rust tests, three benchmark tests,
four governance regressions, cargo fmt --check, all-target Clippy with warnings
denied, and static/current-date strict governance checks. The three new library
tests cover a real WASI command, incompatible source/imports/exports and a core
start function that would trap if instantiated. The new CLI regression confirms
authority checks precede compilation and unsuccessful checks emit no success JSON.

Preflight proves compatibility with this engine configuration at the time of checking. It does not prove successful instantiation, execution, determinism, resource sufficiency or safety against engine defects. Compilation CPU/RSS are outside guest fuel/deadline limits and require an externally bounded worker. The report is unsigned and is not a reusable authorization token: reverify before later execution. No cache artifact, receipt, audit event or package download is produced.

