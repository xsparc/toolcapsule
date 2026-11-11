# Difference csv

Export and verify canonical CSV from a strictly validated retained difference. The existing JSON and HTML outputs are unchanged.

```sh
toolcapsule configuration-diff-csv difference.json --out review.csv
toolcapsule configuration-diff-csv-verify difference.json review.csv --before before.json --after after.json
```

Both commands accept optional paired --before/--after summaries. A partial pair, mismatched sources or reversed direction fails with fixed diagnostics. Source checks precede export or verification. CSV export creates a new file exclusively; verification never writes or repairs evidence.

The version-1 CSV has seven ordered columns: schema_version, direction, field, before, after, change, interpretation. It contains one header and eleven rows in the existing difference-field order. Every cell is quoted, embedded quotes are doubled, encoding is UTF-8 without BOM, and every row ends in CRLF including the last. Data schema_version is 1; all other data cells begin with the literal text: prefix. Before/after values are compact JSON, or not_reported for absent conditional fields. Direction is before_to_after, change is changed/unchanged, and interpretation is summary_changes_not_source_identity_or_authority. Prefixes keep formula markers from leading data cells; removing them changes the canonical contract.

Difference intake is bounded to 16 KiB, explicit summaries to 4096 bytes each, and CSV input/output to 64 KiB. SDK configuration_csv::render and verify share the canonical exporter. Verification requires exact bytes, rejecting edits to fields, cells, ordering, quoting, prefixes, encoding and line endings rather than normalizing them. Success returns 0; invalid input, mismatch or output collision returns 2 with existing fixed JSON diagnostics. Verification prints a fixed success summary.

Consistent replacement of both difference and CSV can pass internal checks; independent summaries detect a different target. Equal summaries remain lossy and establish no original-source identity, grants or package authority. No guest compilation or execution occurs.
