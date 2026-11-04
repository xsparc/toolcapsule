# Check configuration before packaging or execution

```sh
toolcapsule limits-check --limits limits.json
toolcapsule policy-check --policy policy.json
```

Successful checks print version-1 JSON with kind and valid=true, then exit 0. Policy checks also report allowed_tool_count and denies_all_tools. Errors exit 2 without success JSON. Files are read only and input values are not printed. Policy names use the manifest grammar: 1–64 ASCII lowercase letters, digits or hyphens; duplicates are rejected.

Policy validation is an optional authoring check and does not change runtime admission. Deliberately unusable policies such as a zero component cap may still deny all work at runtime but fail this authoring check. Success is not a security audit, package verification or authority grant. ToolCapsule remains stacked on security PR #18.
