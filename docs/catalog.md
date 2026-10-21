# MCP tool catalogs

Use `mcp-catalog` to expose several explicitly trusted local packages:

```json
{"schema_version":1,"packages":[
  {"manifest":"packages/uppercase.json","component":"packages/uppercase.wasm"},
  {"manifest":"packages/format.json","component":"packages/format.wasm"}
]}
```

```sh
toolcapsule mcp-catalog --catalog catalog.json --trust trust.json --policy policy.json --audit audit.jsonl
```

Package paths resolve relative to the catalog file; trust, policy and audit flags
resolve as ordinary CLI paths. Catalog entries are operator configuration, never
guest-supplied paths. Every signed name must be unique and allowed by policy. All
entries pass signature, digest, capability and budget checks before compilation
or serving. Invalid entries abort startup without exposing a partial catalog.

Limits: 64 KiB catalog, 1..16 packages, 64 MiB aggregate source components, and the
existing per-package policy limits. Listing uses lexical signed-name order and
pages of eight. Treat nextCursor as opaque, pass it to tools/list, and stop when
it is absent. Invalid cursors return -32602. Calls select exact names; receipts
retain each tool's signed identity in one audit chain. Calls remain sequential.

The existing `mcp` command still serves one package. Embedders can use
`Server::with_tools` after preparing their own admitted tools. Restart or replace
the server after authority changes; catalog contents are immutable for its life.
No package discovery, network access or automatic refresh occurs. JIT memory,
process isolation and audit rotation remain operator responsibilities.
