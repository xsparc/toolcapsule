# Retained configuration verification

Status: accepted, 2026-11-04. TC021 completes both saved ToolCapsule tasks.

Read combined configuration results within 4 KiB using closed typed structures.
Require integer version 1, kind=configuration and package_admission=not_checked.
Require the correct valid/invalid shape per check: deny-all booleans belong only
to valid policy/trust checks; invalid checks carry a fixed error code. Reject
duplicate and unknown fields. Overall valid must equal all three checks being
valid. Budget comparison requires valid limits and policy; otherwise require
not_checked. fits requires no exceeded fields; exceeds requires a nonempty list
in the producer's fixed order without duplicates. Trust failure does not prevent
budget comparison. A failed configuration can be valid evidence.

configuration-result-check validates retained evidence. configuration-result-verify
recomputes the summary from explicit limits, policy and trust and requires equality.
Both return fixed JSON, exit 0 for consistent evidence and 2 on failure. The
validation schema includes their summaries and diagnostic kind. No inputs are
rewritten; no signing, compilation, admission or execution occurs.

Summary equality is lossy: it does not prove identical policies or keys. A reader
cannot establish the truth of a well-typed deny-all boolean without sources.
The verifier compares it with the newly computed indicator. Matching failure
summaries can verify despite unusable configuration. These checks provide local
consistency, never attestation or package authority.
