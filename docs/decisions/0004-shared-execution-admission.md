# ADR 0004: shared execution admission

Status: accepted

Date: 2026-10-19

TC008 adds `pool::ExecutionPool`, configured with 1..64 slots and cloned across
callers. Single runs reserve one slot; batches reserve their complete active window
atomically before starting. Admission is nonblocking and returns `Error::Busy`
without spawning on saturation. Invalid inputs are rejected before reservation.
Existing unpooled entry points remain available for callers with their own admission.

Batch reservations are conservatively retained until the whole batch and its
owned tasks are dropped. Cancellation keeps the reservation in tasks until the
executor performs cleanup, preventing temporary oversubscription. No internal
waiting queue, background worker, guest authority or replay profile change is added.

Acceptance: shared clones reject saturated work; failed multi-slot reservations
consume nothing; independent pools remain independent; real guest cancellation
holds capacity until cleanup; admitted work retains fresh state and replay behavior.
Run all standard checks and document worker integration and limitations.

Risks: integrators can bypass the pool or create separate pools; authenticated
tenant routing and total process memory remain host responsibilities. Batch
reservations may underutilize slots near completion; predictable admission is
preferred to an unbounded queue or partial batch startup.
