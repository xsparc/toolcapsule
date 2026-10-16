# Contributing

Open an issue describing the problem or propose a small pull request. Explain
the observable behavior, affected trust boundary, and verification performed.
New capabilities and ABI changes require an architecture decision before code.

Follow [development setup](docs/development.md), run the full test suite, and
update architecture, roadmap, and governance evidence when behavior changes.
Use synthetic fixtures. Never include private keys, host paths, or customer data.

Keep adapters thin and host policy separate from signed package requests. A
signature is publisher identity, not permission to access host resources.
Preserve default denial and per-invocation state isolation.

Contributions are licensed under MIT. Agent-assisted changes are welcome;
describe validation and limitations truthfully and preserve author attribution.
