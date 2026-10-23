# Signed audit checkpoints

Status: accepted for implementation, 2026-10-23.

## Decision

Authenticate retained checkpoints using Ed25519 and a caller-selected checkpoint trust store. The version-1 envelope contains key_id, base64 exact checkpoint JSON and a hex signature. Sign the project domain `toolcapsule-audit-checkpoint-v1\0` followed by those bytes; this is ordinary Ed25519 with an application prefix, not Ed25519ctx. Verify the signature before accepting the strict checkpoint contract. Limit envelope bytes to 8192 and decoded checkpoint bytes to 4096. Reject unknown fields, unsupported versions, revoked or wrong keys, malformed signatures and altered payloads. Retain unsigned verification for independently protected anchors. Signed and unsigned CLI anchors are mutually exclusive; exact mode requires either anchor. Exclusive output creation prevents replacement.

## Acceptance

Round-trip authentication, tampering and wrong-domain rejection, revocation, strict schema and size admission, CLI exclusive output, required trust and prefix/exact verification.

## Limits

Signatures authenticate a retained statement, not freshness, completeness after its retained head, trusted time, or physical execution. An old valid signed checkpoint can be replayed; operators must independently retain their latest accepted anchor and enforce freshness. Signing a supplied checkpoint does not revalidate the source log. Key IDs select trusted public keys; aliases for the same key are equivalent. Provision checkpoint custodians separately from package publishers. No remote witness or key-management service is included.

