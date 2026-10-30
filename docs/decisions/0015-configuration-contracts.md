# Configuration contracts

Status: accepted, 2026-10-31. TC020 implements the two ToolCapsule items from the
retained five-task handoff.

Publish a Draft 2020-12 validation schema, embedded in the binary and exported by
validation-schema. Close every object, use fixed discriminators and code enums,
and include one example per error code. The schema describes structural output;
it is not an admission policy or a retained-result semantic verifier.

configuration-check requires explicit limits, policy and trust paths. Read each
independently using the existing byte limits and validators, then compare limits
against policy maxima if both are valid. Return one v1 JSON result on stdout.
Invalid input takes exit precedence (2), valid budget non-fit returns 1, and fit
returns 0. Individual failures use fixed codes, and an unavailable comparison
has status not_checked. Empty policy/trust remain valid deny-all configurations.

No paths, key IDs, keys, input values or nested error chains enter the result.
Checks perform no signing, compilation, execution, package admission or writes.
Reading three files is not an atomic snapshot. Configuration validity cannot
grant authority or identify which package, signer or tool would be accepted.
Existing single-check outputs and default diagnostics remain compatible.

Verification uses real CLI subprocesses for independent errors, fit/non-fit,
deny-all, required arguments, no writes and schema export/discriminator drift.
Existing synthetic signed-WASI acceptance continues to prove runtime denial.
