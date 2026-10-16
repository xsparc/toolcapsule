"""Check ToolCapsule traceability and review freshness using only the stdlib."""
import argparse
import datetime as dt
import json
from pathlib import Path
import tomllib


def check(root, as_of=None):
    findings = []
    try:
        data = tomllib.loads((root / "docs/governance/project-evidence.toml").read_text())
        assert data["schema_version"] == 1 and data["project"] == "toolcapsule"
        assert data["authority"]["dhf_is_authoritative"] is False
        reviewed = dt.date.fromisoformat(data["review"]["last_reviewed"])
        due = dt.date.fromisoformat(data["review"]["review_due"])
        assert due >= reviewed
        if as_of and not reviewed <= as_of <= due:
            findings.append({"severity": "warn", "code": "review.freshness"})
        ids = set()
        for group in ("objectives", "requirements", "work_items"):
            assert data[group], group
            for record in data[group]:
                assert record["id"] not in ids, "duplicate id"
                ids.add(record["id"])
                assert record["status"] in {"active", "planned", "implemented", "closed", "deferred"}
                for key, values in record.items():
                    if key.endswith("_refs"):
                        assert isinstance(values, list) and values, key
                        for value in values:
                            assert isinstance(value, str) and "\\" not in value and ":" not in value
                            path = (root / value).resolve()
                            assert path.is_relative_to(root.resolve()) and path.exists(), value
                if group == "requirements":
                    for field in ("objective_ids", "roadmap_refs", "design_refs", "risk_refs"):
                        assert record.get(field), field
                    if record["status"] == "implemented":
                        for field in ("implementation_refs", "verification_refs", "validation_refs"):
                            assert record.get(field), field
                    if record["status"] == "deferred":
                        assert record.get("deferral_reason") and record.get("reconsideration_trigger")
                if group == "work_items":
                    assert record.get("requirement_ids") and record.get("acceptance_refs")
                    assert record.get("approval_ref", "").startswith("conversation:")
                    if record["status"] == "closed":
                        assert record.get("evidence_refs")
        objective_ids = {r["id"] for r in data["objectives"]}
        requirement_ids = {r["id"] for r in data["requirements"]}
        for record in data["requirements"]:
            assert set(record["objective_ids"]) <= objective_ids
        for record in data["work_items"]:
            assert set(record["requirement_ids"]) <= requirement_ids
    except (OSError, ValueError, KeyError, TypeError, AssertionError) as error:
        findings.append({"severity": "error", "code": "evidence.invalid", "kind": type(error).__name__})
    status = "fail" if any(f["severity"] == "error" for f in findings) else "warn" if findings else "pass"
    return {"schema_version": 1, "status": status, "findings": findings}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--as-of", type=dt.date.fromisoformat)
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()
    result = check(Path(__file__).resolve().parents[1], args.as_of)
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result["status"] == "pass" or (result["status"] == "warn" and not args.strict) else 1)
