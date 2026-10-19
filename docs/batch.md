# Bounded prepared-tool batches

`batch::run_batch(Arc<PreparedTool>, Vec<Invocation>, concurrency)` reuses one
verified component while creating fresh WASI state for every input/seed pair.
The returned results follow input order even when tasks finish out of order.
Guest errors, traps and timeouts remain individual RunResult outcomes; an
invocation infrastructure error aborts siblings and returns an error for the batch.

Admission checks all requests before starting guests: at most 64 invocations,
concurrency 1–16, each signed input limit, aggregate inputs at most 4 MiB, and
worst-case retained stdout plus stderr at most 8 MiB. Output admission is
conservative: two times the manifest output budget times the batch length.
Overflow fails closed. An empty batch with valid concurrency returns no results.

Only the active window is spawned. Dropping the batch future aborts active tasks
cooperatively and drops queued inputs; the Tokio executor must continue polling
to complete cleanup. Each invocation's deadline begins when it starts, not when
it enters the batch. Wrap the whole call in a host timeout for an overall deadline.
On infrastructure failure the scheduler waits for sibling cleanup before returning.

Limits are per call, not global tenant quotas or a total process-memory bound.
Simultaneous callers and guest memories still need an externally bounded worker.
Choose lower concurrency for large memory policies. No cross-tenant policy lookup,
disk cache, native deserialization or ambient authority is added. Replace the
prepared object after revocation as before. Replays keep the existing runtime profile.

The `batch` example accepts the same four files as `embed`, reads one bounded
stdin input, and runs eight fixture seeds with concurrency four, emitting only
ordered audit receipts. A current-thread Tokio runtime provides cooperative
concurrency, not CPU parallelism. No throughput improvement is claimed.
