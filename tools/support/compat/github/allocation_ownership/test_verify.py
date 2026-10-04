"""Validator adversarial laws only; these synthetic data grant no cause credit."""

import copy
import unittest
from common import CASE, FIXTURES
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
