# ADR 0005: verified preparation cache

Status: accepted

Date: 2026-10-20

TC009 follows TC008 under delegated next-scope selection. The recorded benchmark
shows compilation dominates verification and execution for the synthetic guest.
Add an opt-in, process-local LRU of prepared tools, with 1..64 entries and at most
256 MiB retained source bytes. Every lookup verifies the supplied envelope,
component digest, current trust store and current policy before consulting entries.
Identity is the exact signed manifest digest plus signer ID; the manifest binds
component bytes and limits. The cache owns one fixed runtime engine/profile.

Return Arc handles for use with the shared execution pool. Exclusive mutable
access serializes cache operations; the embedding host supplies bounded preparation
workers. Failed verification or compilation leaves existing entries intact.
Least-recently-used eviction and clear drop only cache references. Already returned
handles retain their authority snapshot; the host must revoke those separately.

Acceptance: signature/digest/policy checks on hits, signer/manifest separation,
entry/byte eviction, LRU touch, failed compilation, clear, fresh guest globals and
receipt replay. Extend the synthetic benchmark to measure verified hits. Run the
standard gates. No persisted native artifact deserialization, network fetching,
guest capability change, TTL, background refresh or tenant authentication is added.

Source-byte limits do not bound JIT RSS, peak compilation memory or external Arcs.
Compile before eviction so failure cannot destroy useful cache entries; external
worker resource limits remain required.
