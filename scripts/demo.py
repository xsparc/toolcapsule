"""Build and run a self-contained local demo. Requires Rust and wasm32-wasip2."""
import json
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
local = root / ".local"
local.mkdir(exist_ok=True)
demo = Path(tempfile.mkdtemp(prefix="demo-", dir=local))
subprocess.run(["cargo", "build", "--locked"], cwd=root, check=True)
subprocess.run(["rustc", "--edition", "2024", "--target", "wasm32-wasip2", "-O",
                str(root / "examples/uppercase.rs"), "-o", str(demo / "tool.wasm")], check=True)
binary = root / "target/debug" / ("toolcapsule.exe" if __import__("os").name == "nt" else "toolcapsule")


def run(*args, input=None):
    result = subprocess.run([str(binary), *args], cwd=demo,
                            input=None if input is None else input.encode("utf-8"),
                            capture_output=True, check=True)
    return json.loads(result.stdout) if result.stdout else None


run("keygen", "--key", "signing.key", "--trust", "trust.json")
run("pack", "--component", "tool.wasm", "--name", "uppercase", "--key", "signing.key", "--out", "signed.json")
package = ["--manifest", "signed.json", "--component", "tool.wasm", "--trust", "trust.json",
           "--policy", str(root / "examples/policy.json")]
run("verify", *package)
result = run("run", *package, "--audit", "audit.jsonl", "--receipt", "receipt.json", "--seed", "42",
             input="hello toolcapsule\n")
print(result["stdout"], end="")
print(json.dumps(run("replay", *package, "--receipt", "receipt.json", input="hello toolcapsule\n")))
print(json.dumps(run("audit-verify", "--audit", "audit.jsonl")))
print("Demo files retained under", demo.relative_to(root).as_posix())
