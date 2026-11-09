# Difference review

Render a retained difference after strict decoding and semantic recomputation. The standalone HTML shows all eleven summary fields, before/after values, change labels and explicit evidence limits. Missing conditional fields display Not reported. Configuration failures can be valid review evidence; no package admission is implied.

```sh
toolcapsule configuration-diff-html difference.json --out review.html
toolcapsule configuration-diff-html difference.json --before before.json --after after.json --out checked.html
```

Both optional source-summary arguments must appear together. Partial pairs return a fixed invalid-format diagnostic. A supplied pair is checked against the retained difference before rendering and exclusive output creation; wrong direction or a consistent replacement fails. The page distinguishes internal consistency from matching explicit summaries. It does not print supplied paths.

Difference intake is bounded to 16 KiB, explicit summaries to 4096 bytes each and HTML to 64 KiB. `configuration_review::render` accepts the retained bytes and an optional pair of summary byte slices. It validates before rendering, escapes cells, uses fixed CSS protected by a SHA-256 CSP hash, and permits no scripts or external resources. A focusable scroll region contains the comparison table on narrow screens. Output files are never overwritten; inputs are read-only. Success returns 0, input/output failures return 2 with existing fixed JSON diagnostics.

Summary equality does not establish original configuration identity, grants, signer authority or execution permission. No guest execution or compilation occurs during review. This command exports HTML; it does not validate a subsequently edited HTML file.
