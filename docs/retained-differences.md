# Retained differences

A retained difference is accepted only after strict typed decoding and recomputing the entire envelope from both embedded summaries. The reader rejects duplicate and unknown fields, missing fields, invalid versions and types, contradictory summary states, changed-field omissions/duplicates/order errors, wrong direction and altered interpretation. A legitimate reversed difference is internally valid; checking it against explicit before/after summaries detects the reversal.

```sh
toolcapsule configuration-diff-check difference.json
toolcapsule configuration-diff-verify difference.json before.json after.json
```

Difference files are bounded to 16 KiB before parsing. Explicit summary files remain bounded to 4096 bytes. Embedded summaries are normalized and passed through the existing bounded semantic reader. Equivalent JSON formatting is accepted. Failed configuration summaries remain valid evidence when internally consistent; success here describes verification, not configuration validity.

Both commands emit a fixed version-1 verification summary and return 0 on success, 2 with fixed JSON diagnostics on malformed, mismatched, unreadable or oversized input. They do not execute, compile or write any evidence. `read_difference` and `verify_difference` provide the SDK boundary. The existing difference schema and exporter remain unchanged.

Independent summary files detect replacement of a whole self-consistent envelope. They remain lossy: equality does not establish original configuration content, grants or package authority. Keep independently retained evidence under separate custody; replacing all copies defeats this check.
