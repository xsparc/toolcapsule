# ToolCapsule stewardship

This directory is an independent Git repository. Never change another project
or initialize Git in the parent workspace.

Use the OpenSteward research, delivery, and evidence skills. Read architecture,
roadmap, accepted decisions, and governance evidence before changing behavior.
The authority order is architecture, roadmap, accepted decisions, code and tests,
then supporting documentation. Governance evidence is a derived index.

Keep changes bounded and reviewable. Preserve configured Git attribution.
Do not publish private paths, keys, operator state, or machine configuration.
Use synthetic data in tests. No merging, deployment, releases, paid services,
or contacting people without explicit authorization.

Run cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test,
and the project governance checker before closing a delivery slice. Record
unexecuted checks and limitations truthfully. Never deserialize untrusted native
Wasmtime artifacts. Never inherit ambient filesystem, environment, or network
capabilities into guest components.
