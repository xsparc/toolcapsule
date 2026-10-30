# Check trust files and requested budgets

## Trust validation and budget-fit reporting

```sh
toolcapsule trust-check --trust trust.json
toolcapsule budget-fit --limits limits.json --policy policy.json
```

trust-check exits 0 for a valid store and prints schema_version, kind, valid, key_count and denies_all_signers; empty maps are valid. budget-fit prints fits, exceeded_fields and package_admission=not_checked. It exits 0 when budgets fit, 1 when valid budgets exceed policy, and 2 for invalid input/I/O without report JSON. Limits and policy inputs retain the 4096-byte and 64-KiB caps.

Trust validation proves encoding and non-weakness under the pinned dalek implementation, not ownership, identity or authority. It is an opt-in authoring check; runtime trust parsing is unchanged. Budget fit checks only limits, not signer, capabilities, component size, tool allowlist or package authenticity. The empty tool allowlist can therefore deny a package whose budget fits. Security PR #18 is still the required patched baseline.
