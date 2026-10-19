"""Check ToolCapsule traceability and review freshness using only the stdlib."""
import argparse
import datetime as dt
import json
from pathlib import Path
import tomllib


def _require(condition, message="invalid evidence"):
    # Explicit exceptions remain active under -O/-OO and preserve diagnostics.
    if not condition:
        raise AssertionError(message)


def check(root, as_of=None):
    findings = []
    try:
        data = tomllib.loads((root / "docs/governance/project-evidence.toml").read_text())
        _require(data["schema_version"] == 1 and data["project"] == "toolcapsule")
        _require(data["authority"]["dhf_is_authoritative"] is False)
        reviewed = dt.date.fromisoformat(data["review"]["last_reviewed"])
        due = dt.date.fromisoformat(data["review"]["review_due"])
        _require(due >= reviewed)
        # Maintainer calendar dates may be one day ahead of a UTC CI runner.
        if as_of and (reviewed > as_of + dt.timedelta(days=1) or as_of > due):
            findings.append({"severity": "warn", "code": "review.freshness"})
        ids = set()
        for group in ("objectives", "requirements", "work_items"):
            _require(data[group], group)
            for record in data[group]:
                _require(record["id"] not in ids, "duplicate id")
                ids.add(record["id"])
                _require(record["status"] in {"active", "planned", "implemented", "closed", "deferred"})
                for key, values in record.items():
                    if key.endswith("_refs"):
                        _require(isinstance(values, list) and values, key)
                        for value in values:
                            _require(isinstance(value, str) and "\\" not in value and ":" not in value)
                            path = (root / value).resolve()
                            _require(path.is_relative_to(root.resolve()) and path.exists(), value)
                if group == "requirements":
                    for field in ("objective_ids", "roadmap_refs", "design_refs", "risk_refs"):
                        _require(record.get(field), field)
                    if record["status"] == "implemented":
                        for field in ("implementation_refs", "verification_refs", "validation_refs"):
                            _require(record.get(field), field)
                    if record["status"] == "deferred":
                        _require(record.get("deferral_reason") and record.get("reconsideration_trigger"))
                if group == "work_items":
                    _require(record.get("requirement_ids") and record.get("acceptance_refs"))
                    _require(record.get("approval_ref", "").startswith("conversation:"))
                    if record["status"] == "closed":
                        _require(record.get("evidence_refs"))
        objective_ids = {r["id"] for r in data["objectives"]}
        requirement_ids = {r["id"] for r in data["requirements"]}
        for record in data["requirements"]:
            _require(set(record["objective_ids"]) <= objective_ids)
        for record in data["work_items"]:
            _require(set(record["requirement_ids"]) <= requirement_ids)
    except (OSError, ValueError, KeyError, TypeError, AssertionError) as error:
        findings.append({"severity": "error", "code": "evidence.invalid", "kind": type(error).__name__})
    status = "fail" if any(f["severity"] == "error" for f in findings) else "warn" if findings else "pass"
    return {"schema_version": 1, "status": status, "findings": findings}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    dates = parser.add_mutually_exclusive_group()
    dates.add_argument("--as-of", type=dt.date.fromisoformat)
    dates.add_argument("--today", action="store_true", help="check freshness using the current UTC date")
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()
    as_of = dt.datetime.now(dt.timezone.utc).date() if args.today else args.as_of
    result = check(Path(__file__).resolve().parents[1], as_of)
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result["status"] == "pass" or (result["status"] == "warn" and not args.strict) else 1)
