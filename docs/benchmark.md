# Prepared execution benchmark

Compile examples/guest.rs as described in development.md, then run:

```sh
cargo run --locked --release --example benchmark -- target/fixtures/guest.wasm --samples 10 --warmup 3
cargo test --locked --example benchmark
```

JSON stdout separates engine setup, signature/digest verification (including a
component copy), preparation (compilation/import resolution), first invocation of
each newly prepared tool and reuse of the last tool. Disk reads, signing and JSON
serialization are excluded. Preparation uses one engine in one process, not cold
processes or cold filesystem caches. Reuse still creates fresh guest state.

Every run must uppercase the synthetic input, produce no stderr, succeed and use
identical fuel; otherwise no timing report is published. Timings are not replay
evidence. Samples are bounded to 1–100 and warmups to 0–100. Percentiles use nearest
rank without interpolation. Small samples do not establish tail-latency guarantees.

Reports identify schema, runtime profile, build mode, component digest/size, input
size, seed and nanosecond summaries. They omit host identity, paths and guest output.
The fixture signing key is public test data; never trust it in an operator policy.
Use only the locally compiled fixture. Compilation needs external process limits.
Compare repeated release-mode measurements before tuning. No cache, pooling,
parallel throughput, RSS guarantee or service SLA is introduced.
