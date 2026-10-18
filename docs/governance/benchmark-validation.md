# TC005 validation

2026-10-18: all 16 existing tests and three benchmark tests passed. Formatting and
Clippy with warnings denied passed. A five-sample debug fixture run passed output
and fuel checks (26,292 fuel each). Debug timings are functional evidence only.
The release run passed ten samples per phase plus three reuse warmups. The saved
benchmark-sample.json records a 2,840,199-byte component, 26,292 fuel each run,
and median times of 1.574 ms verification/copy, 195.9901 ms preparation,
0.1572 ms first invocation and 0.0916 ms reuse. These are one Windows run with
nearest-rank statistics, not a speedup guarantee or a cold-process measurement.
Static and dated strict governance checks passed. The sample intentionally omits
machine identity and paths; use it as format/correctness evidence, not a portable
hardware comparison. Remote CI results remain separate.

No parallel throughput, memory/RSS, independent security review or cross-platform
performance claim. CI adds a one-sample correctness smoke, not timing thresholds.
