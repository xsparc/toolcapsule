# Retain and verify audit checkpoints

An internally valid audit chain can still have been rewritten. Retain a trusted
count and chain head outside the writable log, then check that history explicitly:

```sh
toolcapsule audit-checkpoint --audit audit.jsonl --out checkpoint.json
toolcapsule audit-verify --audit audit.jsonl --checkpoint checkpoint.json
```

Export verifies every entry before exclusively writing a version-1 checkpoint:
`schema_version`, `entries` and `checkpoint`. It also prints the checkpoint JSON.
Existing destinations are never overwritten. Store the exported file in a separately
protected location and supply that retained copy on subsequent verification.

The default anchored check permits valid appended entries but requires the retained
prefix to match. Add `--exact` to require exactly the retained count as well.
Successful verification keeps the existing entries/checkpoint JSON response; failures
exit 2 with sanitized diagnostics. Without either checkpoint option, audit-verify
only checks internal consistency. --exact requires an unsigned or signed anchor.

The library provides `checkpoint_log` and `verify_log_against`. Both verify the full
log within the existing 8 MiB cap. Checkpoint CLI input is capped at 4 KiB and uses
a closed schema. An empty log produces zero entries and an empty hash: it anchors
no history. Nonempty checkpoints require a lowercase SHA-256 chain-head hash.

Exported checkpoints are unsigned unless authenticated as described below.
Replacing both log and an unprotected unsigned checkpoint defeats this check.
Changes after the anchor are unprotected until a newer trusted checkpoint is
retained; exact mode detects any suffix change by refusing suffixes entirely.
These checks establish logical chain consistency, not byte-for-byte JSON identity,
signer authenticity or execution truth. Stop writers for a stable exported snapshot;
a concurrent partial append can make verification fail and should be retried after
writers are quiescent. Log rotation and external retention remain operator actions.

## Authenticate retained checkpoints

Create a dedicated custodian key and trust file with `toolcapsule keygen`.
Keep private keys out of repositories and distribute the public trust file through
an independent trusted channel. Package-publisher trust is not automatically
checkpoint-custodian trust.

```sh
toolcapsule audit-checkpoint --audit audit.jsonl --out anchor.json
toolcapsule audit-checkpoint-sign --checkpoint anchor.json --key custodian.key --out anchor.signed.json
toolcapsule audit-verify --audit audit.jsonl --signed-checkpoint anchor.signed.json --trust custodians.json
```

The default key ID is `local`; use matching `--key-id` values when generating and
signing with another ID. Add `--exact` to reject appended entries as well. Outputs
are created exclusively. Verification requires current explicit trust and rejects
revoked keys; `--checkpoint` and `--signed-checkpoint` cannot be combined.

Signatures authenticate the checkpoint bytes, not freshness or physical execution.
Retain the latest accepted checkpoint independently: an old valid signed anchor
can be replayed. Signing supplied JSON does not reread or validate its original log.
The envelope is limited to 8192 bytes and decoded checkpoint JSON to 4096 bytes.
See [ADR 0008](decisions/0008-signed-audit-checkpoints.md) for the wire contract.
