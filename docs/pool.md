# Shared execution limits

Create one `pool::ExecutionPool::new(16)?` for a worker and clone it into request
handlers. Use `pool.run(&tool, input, seed).await` or
`pool.run_batch(tool, invocations, concurrency).await`. Share the same pool across
prepared tools to enforce a worker-wide ceiling. Separate pools have separate limits.

Capacity must be 1..64. Saturation returns `Error::Busy` immediately; the caller
decides whether to reject or retry. There is no waiting queue or automatic retry.
A batch reserves `min(input_count, concurrency)` slots for its entire lifetime;
it starts all-or-nothing and still obeys the batch count/byte limits. A request
larger than pool capacity always returns Busy. An empty valid batch needs no slots.

Dropping a single run releases its store and slot directly. Dropping a batch aborts
its tasks; slots remain reserved until the executor drops those tasks. Keep polling
the executor during shutdown. Guest fuel, timeout, fresh state, denied capabilities
and receipts follow the existing runtime contract. Compilation is outside the pool.

This API limits participating invocations, not process RSS, retained results,
requests held by the caller, rate, tenants or other processes. Authenticate callers,
select the correct trust/policy snapshot, bound upstream requests and isolate workers.
Direct `PreparedTool::run` and `batch::run_batch` bypass this optional shared pool.
