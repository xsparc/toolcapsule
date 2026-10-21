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
exit 2 with sanitized diagnostics. Without --checkpoint, audit-verify only checks
internal consistency, as before. --exact requires --checkpoint.

The library provides `checkpoint_log` and `verify_log_against`. Both verify the full
log within the existing 8 MiB cap. Checkpoint CLI input is capped at 4 KiB and uses
a closed schema. An empty log produces zero entries and an empty hash: it anchors
no history. Nonempty checkpoints require a lowercase SHA-256 chain-head hash.

Checkpoints are unsigned. Replacing both log and checkpoint defeats this check.
Changes after the anchor are unprotected until a newer trusted checkpoint is
retained; exact mode detects any suffix change by refusing suffixes entirely.
These checks establish logical chain consistency, not byte-for-byte JSON identity,
signer authenticity or execution truth. Stop writers for a stable exported snapshot;
a concurrent partial append can make verification fail and should be retried after
writers are quiescent. Log rotation and external retention remain operator actions.
