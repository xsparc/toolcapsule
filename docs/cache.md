# Verified preparation cache

Create `cache::PreparationCache::new(8, 32 * 1024 * 1024)?`, then call
`cache.prepare(envelope, component_bytes, &trust, &policy)?` for an `Arc<PreparedTool>`.
Each call verifies current signature, digest and policy, including a cache hit.
Use the returned handle with `ExecutionPool` or the existing runtime APIs.

The cache holds up to the configured number of entries and source bytes. Bounds
are 1..64 entries and 1..256 MiB; a package larger than the cache budget is denied.
Successful hits move an entry to the most-recent position. New successful
preparations evict from the least-recent end. Errors do not evict entries.
`len`, `is_empty`, `source_bytes` and `clear` expose basic cache management.

Preparation is synchronous and requires mutable access. Route compilation through
a bounded host worker. There is no cache-owned thread, async wait queue, disk cache
or native artifact deserialization. A cache has one fixed runtime profile.

Previously returned handles survive eviction, clear and trust changes. Replace or
drop those handles separately when revoking authority. Fresh lookups always use
the trust and policy supplied to that call. This API is not a tenant registry or
authentication boundary. Compiled memory, transient copies, compilation peaks and
external handles need separate process limits.

The benchmark reports `verified_cache_hit`, including input cloning and current
verification. It checks pointer reuse and guest correctness before publishing
timings. Existing preparation and execution phase measurements remain available.
