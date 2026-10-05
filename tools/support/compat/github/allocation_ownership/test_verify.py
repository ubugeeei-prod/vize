"""Validator adversarial laws only; these synthetic data grant no cause credit."""

import copy
import hashlib
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock
from common import BRANCH, CASE, FIXTURES
from run import experiment, prepare_output, verify_budget_overlay
from verify import budget_trace, printed_rows


def synthetic():
    rows, windows, events, ordinal = [], [], [], 100
    for index, (name, cap) in enumerate(FIXTURES):
        native = 78 if index == 0 else cap
        rows.append({"fixture": name, "ceiling": cap, "native": native, "retained": cap})
        for lane, count in enumerate((native, cap)):
            start = ordinal
            for offset in range(count):
                ordinal += 1
                events.append({"ordinal": ordinal, "tid": 10 if index == lane == 0 and offset < 3 else 11,
                               "size": 24, "operation": 1})
            windows.append({"context": index * 2 + lane, "calls_start": start,
                            "calls_end": ordinal, "measuring_tid": 11})
            ordinal += 20
    trace = {"schema_version": 1, "status": "diagnostic-observed", "process_pid": 10,
             "ledger_capacity": 8192, "overflow": False, "unpublished_events": 0,
             "in_flight_events": 0, "errors": [], "windows": windows, "events": events}
    return rows, trace


class EvidenceValidator(unittest.TestCase):
    def test_changed_libtest_overlay_refused_before_external_work(self):
        # Positive bytes and pin here are an authored guard control, not a
        # recovered historical runtime or permission to execute the campaign.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            budget = root / "crates/vize_atelier_vapor/tests/davinci_vapor_native_budget.rs"
            budget.parent.mkdir(parents=True)
            original = b"authored guard control\n"
            budget.write_bytes(original)
            with mock.patch("run.LIBTEST_BUDGET_OVERLAY_SHA256", hashlib.sha256(original).hexdigest()):
                verify_budget_overlay(root)
                budget.write_bytes(original + b"changed\n")
                with self.assertRaisesRegex(RuntimeError, "overlay changed"):
                    verify_budget_overlay(root)
            actual_root = Path(__file__).resolve().parents[5]
            with self.assertRaisesRegex(RuntimeError, "overlay changed"):
                verify_budget_overlay(actual_root)
            env = {"GITHUB_REF": f"refs/heads/{BRANCH}", "GITHUB_EVENT_NAME": "workflow_dispatch"}
            with mock.patch("run.sys.platform", "linux"), mock.patch.dict(os.environ, env, clear=True), \
                 mock.patch("run.command", side_effect=AssertionError("no tool process")), \
                 mock.patch("run.git", side_effect=AssertionError("no git process")), \
                 mock.patch("run.original_archive", side_effect=AssertionError("no archive download")), \
                 mock.patch("run.subprocess.run", side_effect=AssertionError("no external process")):
                with self.assertRaisesRegex(RuntimeError, "overlay changed"):
                    experiment(actual_root, root)
            self.assertFalse((root / "identity.json").exists())

    def test_fresh_preparation_admitted_and_prior_matrix_or_wrong_identity_refused(self):
        env = {"GITHUB_SHA": "frozen-head", "GITHUB_RUN_ID": "run-id", "GITHUB_RUN_ATTEMPT": "1"}
        value = {"status": "preparation-only", "matrix_execution_credit": False,
                 "matrix_attempts": 0, "head_sha": "frozen-head", "run_id": "run-id", "run_attempt": "1"}
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            receipt = output / "preparation.json"
            receipt.write_text(json.dumps(value))
            prepare_output(output, env)
            self.assertEqual(json.loads(receipt.read_text()), value)
            (output / "identity.json").write_text("{}")
            with self.assertRaises(RuntimeError):
                prepare_output(output, env)
            (output / "identity.json").unlink()
            for key, wrong in (("head_sha", "other"), ("run_id", "other"),
                               ("run_attempt", "2"), ("matrix_attempts", 1),
                               ("matrix_execution_credit", True), ("matrix_execution_credit", 0),
                               ("matrix_attempts", False)):
                receipt.write_text(json.dumps({**value, key: wrong}))
                with self.assertRaises(RuntimeError):
                    prepare_output(output, env)

    def test_exact_main_identity_join_and_original_cap_failure(self):
        rows, trace = synthetic()
        joined = budget_trace(trace, rows)
        self.assertEqual((joined[0]["global_calls"], joined[0]["measuring_calls"],
                          joined[0]["libtest_main_calls"]), (78, 75, 3))
        stdout = f"test {CASE} ... " + "\n".join(
            f"measured {row['fixture']}: native {row['native']} retained {row['retained']}"
            for row in rows) + "\ntest result: FAILED. 0 passed; 1 failed;\n"
        self.assertEqual(printed_rows(stdout, "text_runs: 78 > ceiling 75", 101), rows)
        with self.assertRaises(RuntimeError):
            printed_rows(stdout, "text_runs: 78 > ceiling 75", 0)

    def test_completeness_and_identity_adversaries_are_fatal(self):
        rows, original = synthetic()
        mutations = [
            lambda t: t["windows"].clear(),
            lambda t: t["windows"].pop(),
            lambda t: t["windows"][1].update(context=0),
            lambda t: t["windows"][1].update(calls_start=0),
            lambda t: t["windows"][0].update(measuring_tid=0),
            lambda t: t["events"].pop(0),
            lambda t: t["events"].append(copy.deepcopy(t["events"][0])),
            lambda t: t["events"][0].update(tid=2**64 - 1),
            lambda t: t.update(in_flight_events=1),
            lambda t: t.update(unpublished_events=1),
            lambda t: t.update(overflow=True),
            lambda t: t.update(status="diagnostic-invalid"),
        ]
        for mutation in mutations:
            trace = copy.deepcopy(original)
            mutation(trace)
            with self.assertRaises(RuntimeError):
                budget_trace(trace, rows)
        wrong_rows = copy.deepcopy(rows)
        wrong_rows[0]["native"] -= 1
        with self.assertRaises(RuntimeError):
            budget_trace(original, wrong_rows)


if __name__ == "__main__":
    unittest.main()
