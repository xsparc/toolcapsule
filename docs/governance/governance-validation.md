# TC006 validation

Scope: keep the existing governance contract active under Python optimization.
Python removes assert statements under -O and -OO, so those statements cannot
enforce evidence validation. See the
[Python language reference](https://docs.python.org/3/reference/simple_stmts.html#the-assert-statement).

Baseline 72e4f49 rejected an invalid schema normally but returned pass when
optimized. Four CLI regression tests reproduced 26 invalid-evidence passes
across -O/-OO before the correction. The fixtures contain synthetic metadata and
an existing file outside their repository to exercise reference containment.

Acceptance: valid registries pass in all three modes; invalid schema/project,
authority, status, references, required evidence, traceability, duplicate IDs,
empty objectives and missing deferral rationale fail. Expired reviews retain
warning behavior, strict mode rejects warnings, static checks remain undated,
and malformed TOML remains a sanitized failure. Public JSON fields and exit-code
semantics remain unchanged.

2026-10-19 local validation: all four governance tests passed in the three
execution modes, including preserved failure categories. All 16 existing
package/runtime/CLI tests and three benchmark tests passed. Formatting,
Clippy with warnings denied, static governance and dated strict checks passed.
The refreshed locked-dependency RustSec audit reported no advisories.
Hosted matrix results are tracked separately by the draft PR.

Tests use only the Python standard library.
The runtime, cryptography, package policy, guest ABI and replay profile are outside
this correction. The checker is a maintenance gate, not a sandbox boundary.
