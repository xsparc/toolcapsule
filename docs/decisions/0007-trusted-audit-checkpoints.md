# ADR 0007: trusted audit checkpoint verification

Status: accepted

Date: 2026-10-22

TC011 follows the next-milestone request with delegated research and scope selection.
TC010's catalog PR remains open; this change starts from main and is independent.
The existing audit verifier checks internal consistency, but a valid rewritten or
truncated chain also passes. Add an explicit unsigned checkpoint contract holding
schema_version 1, entries and the existing chain-head checkpoint hash.

The export command first verifies the complete bounded log and exclusively creates
a new checkpoint file. Verification accepts an externally retained checkpoint and
requires its count/hash to match the corresponding prefix while still verifying
all later entries. Exact mode additionally requires equal entry counts. Reject
unsupported checkpoint versions, unknown fields, invalid counts/hash syntax,
shorter logs, altered prefixes and malformed suffixes. Preserve the existing
unanchored audit-verify output and behavior.

Acceptance: exact/extended chains, internally valid rewrites, truncation, corrupt
suffixes, empty checkpoint semantics, strict schema and 4 KiB checkpoint input;
real CLI export/verify, non-overwrite and no output on invalid logs. Existing
runtime, replay, CLI and governance gates continue to pass.

The empty checkpoint is count zero with an empty hash and establishes no history.
A nonempty checkpoint uses 64 lowercase hexadecimal characters. Protect retained
checkpoints separately from writable logs. Checkpoint export is a local observation,
not a signature, timestamp, signer verification or proof of actual execution.
A prefix check cannot detect deletion of events after its retained anchor. Exact
mode rejects all append operations. No remote witness, automatic rotation or new
audit encoding is introduced; the existing 8 MiB cap remains.
