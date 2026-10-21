# DirectionBriefV1: multi-tool MCP integration

- schema_version: 1
- baseline_commit: 5cedfd561ddf8fc243db8dd7989ca38da9e763c0
- scanned_at: 2026-10-21
- coverage: merged TC009, MCP adapter and admission boundaries, primary MCP tools
  specification, existing cache/pool interfaces and deferred capabilities.

Finding: the current CLI serves only one signed tool per process. The pinned
[MCP tools contract](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
supports uniquely named tools and paginated listing. A static catalog can reduce
integration process count while preserving explicit admission and tool identity.
source_event_date: specification version 2025-11-25, accessed 2026-10-21;
confidence: high; relevance: multi-tool agent clients; direction_alignment:
framework-independent verified tools; opportunity_or_risk: simpler integration
versus ambiguous names and aggregate compilation cost; engineering_cost: medium;
disposition: adopt; roadmap_delta: TC010 static bounded catalog;
verification_needed: all-or-nothing admission, ordering, pagination, routing and
receipt identity. This is not a claim that the pinned protocol is the latest.

Overall recommendation: deliver the catalog before adding guest host authority.
WASI ABI migration needs a separate compatibility fixture; brokered I/O needs an
explicit capability and isolation design. Evidence gaps: no external client UI
conformance or multi-tenant security assessment; real stdio tests cover the adapter.
