"""Exercise governance validation through the real CLI and Python optimization flags."""
from copy import deepcopy
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


CHECKER = Path(__file__).with_name("check_governance.py")
VALID = {
    "schema_version": 1,
    "project": "toolcapsule",
    "authority": {"dhf_is_authoritative": False},
    "review": {"last_reviewed": "2026-10-19", "review_due": "2026-11-19"},
    "objectives": [{
        "id": "OBJ-SYNTHETIC", "statement": "Synthetic objective", "status": "active",
        "source_refs": ["evidence.md"], "validation_refs": ["evidence.md"],
    }],
    "requirements": [{
        "id": "REQ-SYNTHETIC", "statement": "Synthetic requirement", "status": "implemented",
        "objective_ids": ["OBJ-SYNTHETIC"], "roadmap_refs": ["evidence.md"],
        "design_refs": ["evidence.md"], "risk_refs": ["evidence.md"],
        "implementation_refs": ["evidence.md"], "verification_refs": ["evidence.md"],
        "validation_refs": ["evidence.md"],
    }],
    "work_items": [{
        "id": "TC-SYNTHETIC", "title": "Synthetic work", "status": "closed",
        "requirement_ids": ["REQ-SYNTHETIC"], "acceptance_refs": ["evidence.md"],
        "evidence_refs": ["evidence.md"], "approval_ref": "conversation:synthetic-test",
    }],
}


def serialize(payload):
    lines = [f"schema_version = {json.dumps(payload['schema_version'])}",
             f"project = {json.dumps(payload['project'])}"]
    for group in ("authority", "review"):
        lines.append(f"\n[{group}]")
        lines.extend(f"{key} = {json.dumps(value)}" for key, value in payload[group].items())
    for group in ("objectives", "requirements", "work_items"):
        if not payload[group]:
            # Empty arrays must precede table declarations to remain root-level.
            lines.insert(0, f"{group} = []")
        for record in payload[group]:
            lines.append(f"\n[[{group}]]")
            lines.extend(f"{key} = {json.dumps(value)}" for key, value in record.items())
    return "\n".join(lines) + "\n"


class GovernanceCliTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="toolcapsule-governance-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "repository"
        self.registry = self.root / "docs/governance/project-evidence.toml"
        self.registry.parent.mkdir(parents=True)
        (self.root / "scripts").mkdir()
        shutil.copyfile(CHECKER, self.root / "scripts/check_governance.py")
        (self.root / "evidence.md").write_text("Synthetic evidence\n", encoding="utf-8")
        (self.root.parent / "outside.md").write_text("Outside repository\n", encoding="utf-8")

    def run_checker(self, payload, optimization=(), *args):
        self.registry.write_text(serialize(payload), encoding="utf-8")
        result = subprocess.run(
            [sys.executable, "-B", *optimization,
             str(self.root / "scripts/check_governance.py"), *args],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.stderr, "")
        output = json.loads(result.stdout)
        self.assertNotIn(str(self.root), result.stdout)
        return result.returncode, output

    def test_valid_evidence_passes_in_every_optimization_mode(self):
        for mode in ((), ("-O",), ("-OO",)):
            with self.subTest(mode=mode):
                code, result = self.run_checker(VALID, mode, "--as-of", "2026-10-19", "--strict")
                self.assertEqual((code, result["status"]), (0, "pass"))

    def test_invalid_evidence_fails_in_every_optimization_mode(self):
        cases = {}
        for name in ("schema", "project", "authority", "status", "missing_reference",
                     "escaping_reference", "missing_verification", "missing_closed_evidence",
                     "dangling_objective", "dangling_requirement", "duplicate_id",
                     "empty_objectives", "deferred_without_reason"):
            cases[name] = deepcopy(VALID)
        cases["schema"]["schema_version"] = 999
        cases["project"]["project"] = "different-project"
        cases["authority"]["authority"]["dhf_is_authoritative"] = True
        cases["status"]["requirements"][0]["status"] = "unsupported"
        cases["missing_reference"]["objectives"][0]["source_refs"] = ["missing.md"]
        cases["escaping_reference"]["objectives"][0]["source_refs"] = ["../outside.md"]
        del cases["missing_verification"]["requirements"][0]["verification_refs"]
        del cases["missing_closed_evidence"]["work_items"][0]["evidence_refs"]
        cases["dangling_objective"]["requirements"][0]["objective_ids"] = ["OBJ-MISSING"]
        cases["dangling_requirement"]["work_items"][0]["requirement_ids"] = ["REQ-MISSING"]
        cases["duplicate_id"]["work_items"][0]["id"] = "OBJ-SYNTHETIC"
        cases["empty_objectives"]["objectives"] = []
        cases["deferred_without_reason"]["requirements"][0]["status"] = "deferred"
        for name, payload in cases.items():
            for mode in ((), ("-O",), ("-OO",)):
                with self.subTest(case=name, mode=mode):
                    code, result = self.run_checker(payload, mode, "--as-of", "2026-10-19", "--strict")
                    self.assertNotEqual(code, 0)
                    self.assertEqual(result["status"], "fail")
                    self.assertEqual(result["findings"][0]["code"], "evidence.invalid")
                    self.assertEqual(result["findings"][0]["kind"], "AssertionError")

    def test_expired_review_remains_a_strict_warning(self):
        for mode in ((), ("-O",), ("-OO",)):
            with self.subTest(mode=mode):
                code, result = self.run_checker(VALID, mode, "--as-of", "2026-11-20")
                self.assertEqual((code, result["status"]), (0, "warn"))
                code, result = self.run_checker(VALID, mode, "--as-of", "2026-11-20", "--strict")
                self.assertEqual((code, result["status"]), (1, "warn"))
                code, result = self.run_checker(VALID, mode)
                self.assertEqual((code, result["status"]), (0, "pass"))

    def test_malformed_toml_is_a_sanitized_failure(self):
        self.registry.write_text("schema_version = [", encoding="utf-8")
        for mode in ((), ("-O",), ("-OO",)):
            with self.subTest(mode=mode):
                result = subprocess.run(
                    [sys.executable, "-B", *mode,
                     str(self.root / "scripts/check_governance.py")],
                    capture_output=True, text=True, check=False,
                )
                self.assertEqual(result.returncode, 1)
                self.assertEqual(json.loads(result.stdout)["status"], "fail")
                self.assertEqual(result.stderr, "")
                self.assertNotIn(str(self.root), result.stdout)


if __name__ == "__main__":
    unittest.main()
