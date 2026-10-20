# TC009 verification

Four cache regressions cover exact prepared-object reuse, fresh globals/replay,
revoked and rotated signers, tampering, changed policy, manifest/signer separation,
LRU touch, source/entry bounds, failed compilation and externally retained handles.
The benchmark also checks cached object identity and fixture correctness.

Local verification passed: 31 core/integration tests, three benchmark tests,
four governance regressions, formatting, all-target Clippy and strict governance.

The release-build uppercase fixture used 10 samples after three warmups. Median
preparation was 185.2882 ms; verified cache-hit lookup, including source cloning
and current package verification, was 1.4918 ms. Each hit reused the same prepared
object and its invocation matched the uncached result and fuel usage. The measured
sample is `cache-sample.json`; it is a synthetic observation, not a service SLA.

No disk/native-code cache, cloud service, cross-tenant authentication or compiled
RSS limit is claimed. Existing returned Arc handles survive authority changes.
