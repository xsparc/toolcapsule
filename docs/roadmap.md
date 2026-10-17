# Roadmap

The initial authorized delivery is bounded to three dependent PRs. Bootstrap
main contains only repository metadata. PRs remain open pending maintainer review.

| Slice | Outcome | Acceptance |
| --- | --- | --- |
| TC001 | Signed packages and architecture | Signature, digest, strict-schema, capability and budget admission tests; governance gate |
| TC002 | Constrained WASI runtime and replay | Real component execution, denied I/O, resource exhaustion, fresh state, audit and replay tests |
| TC003 | CLI, MCP, examples and CI | End-to-end signing/execution/replay; MCP lifecycle and error tests; portable CI |

Later work requires a bounded accepted scope: WASI 0.3 async ABI; brokered
read-only filesystem and domain-restricted HTTP; external secret handles;
tenant authentication and quotas; signed registry distribution; independent
sandbox review; measured cache and throughput tuning. Subprocess access is
deferred until an independently isolated broker exists.

Ongoing stewardship reviews dependency/security changes, CI failures, open PRs,
and user needs. It produces proposals and bounded reviewable changes. It never
authorizes merges, deployment, releases, paid services, or contacting people.
