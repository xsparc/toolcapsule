# Development

Install Rust 1.96 or newer and the WASI target, then build the real fixture before
running tests. The fixture is generated locally, never downloaded as a binary.

```sh
rustup target add wasm32-wasip2
mkdir -p target/fixtures
rustc --edition 2024 --target wasm32-wasip2 -O examples/guest.rs -o target/fixtures/guest.wasm
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python scripts/check_governance.py --as-of 2026-10-17 --strict
```

PowerShell users can replace the mkdir command with
`New-Item -ItemType Directory -Force target/fixtures`.

Runtime tests exercise synthetic WAT components and a real Rust WASI command.
The guest attempts filesystem, environment, TCP, UDP, and subprocess access,
reads the fixed clock, exceeds output capacity, and blocks in a timer. None of
the tests intentionally accesses real host data or external services.

The async runtime uses deterministic fuel metering with cooperative yields every
10,000 fuel units and a Tokio timeout around instantiation and execution. This
also cancels blocked WASI host futures. Compilation is excluded; admission must
run in an externally constrained worker for untrusted publishers.
