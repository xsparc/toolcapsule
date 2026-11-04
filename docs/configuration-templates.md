# Generate configuration templates

```sh
toolcapsule limits-template --out limits.json
toolcapsule policy-template --limits limits.json --out policy.json
```

Omit `--out` to print JSON. `policy-template` without `--limits` uses the default budgets. The generated `allowed_tools` array is empty; explicitly edit it before admitting a tool. The component size cap remains 16 MiB. `limits-template` supplies the complete input shape accepted by `pack --limits`. Neither command creates signing keys or configures trust.

Templates are starting points, not authority grants or workload recommendations. Operators must choose appropriate budgets and allow tools explicitly. Existing signature, trust, capability and runtime enforcement remain in force. This milestone is stacked on security PR #18, including the signed-budget work merged there in #19.
