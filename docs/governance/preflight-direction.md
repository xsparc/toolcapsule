# DirectionBriefV1: Package preflight

- schema_version: 1
- baseline_commit: 5fb9e69ff39243ffa581119111cabafb306cbbb7
- scanned_at: 2026-10-25
- coverage: merged milestone, roadmap, architecture, current CLI and library
  boundaries, regression tests, open draft status and primary platform guidance.

Finding: The current verify command accepts authenticated bytes without compiling them. Operators otherwise need to attempt execution to discover malformed components or incompatible command interfaces. Wasmtime documents synchronous component validation/compilation separately from instantiation. The existing pinned runtime already exposes preparation, so a preflight adapter closes this gap without introducing a new execution path.

Source: [Wasmtime Component documentation](https://docs.wasmtime.dev/api/wasmtime/component/struct.Component.html).
source_event_date: undated living documentation, accessed 2026-10-25;
confidence: high for the documented boundary, subject to local pinned-version tests;
relevance: package preflight; direction_alignment: portable verifiable workflows;
opportunity_or_risk: earlier incompatibility detection without implicit execution,
balanced against compilation resource use; engineering_cost: low;
disposition: adopt; roadmap_delta: TC013;
verification_needed: Real WASI fixture identity and profile; malformed source, absent command export and unresolved imports fail; valid signed invalid source distinguishes verify from preflight; trust/digest denial precedes compilation; trapping start and command code are not run.

Overall recommendation: implement this bounded usability gap on current main.
ToolCapsule ABI/broker changes require a wider capability contract; its catalog
remains a separate draft. Those candidates need separate acceptance criteria.
Evidence gaps: Preflight proves compatibility with this engine configuration at the time of checking. It does not prove successful instantiation, execution, determinism, resource sufficiency or safety against engine defects. Compilation CPU/RSS are outside guest fuel/deadline limits and require an externally bounded worker. The report is unsigned and is not a reusable authorization token: reverify before later execution. No cache artifact, receipt, audit event or package download is produced.

