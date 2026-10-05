"""Reject incomplete observations; a recorded cap failure remains a failure."""

import re
from common import CASE, FIXTURES, require


def printed_rows(stdout, stderr, exit_code):
    rows = re.findall(r"measured (\w+): native (\d+) retained (\d+)(?:\r?\n|$)",
                      stdout, re.MULTILINE)
    require([row[0] for row in rows] == [name for name, _ in FIXTURES],
            "exactly all seven original ordered rows required")
    result = [{"fixture": name, "ceiling": ceiling, "native": int(native),
               "retained": int(retained)}
              for (name, native, retained), (_, ceiling) in zip(rows, FIXTURES)]
    over = [f"{row['fixture']}: {row['native']} > ceiling {row['ceiling']}"
            for row in result if row["native"] > row["ceiling"]]
    if over:
        require(exit_code == 101 and "; ".join(over) in stderr,
                "unchanged original cap assertion must fail with all failures")
        require(CASE in stdout and "test result: FAILED. 0 passed; 1 failed;" in stdout,
                "libtest failed case required")
    else:
        require(exit_code == 0, "below-cap original law must exit successfully")
        require(CASE in stdout and "test result: ok. 1 passed; 0 failed;" in stdout,
                "libtest passed case required")
    return result


def exact_windows(trace, expected_contexts):
    require(trace["schema_version"] == 1 and trace["status"] == "diagnostic-observed",
            "complete diagnostic receipt required")
    require(trace["errors"] == [] and not trace["overflow"] and
            trace["unpublished_events"] == 0 and trace["in_flight_events"] == 0,
            "overflow, incomplete publication or pending writer is fatal")
    pid = trace["process_pid"]
    require(isinstance(pid, int) and 0 < pid < 2**31, "actual positive Linux PID required")
    windows = trace["windows"]
    require([window["context"] for window in windows] == list(expected_contexts),
            "all exact ordered unique contexts required")
    events = trace["events"]
    require(len(events) <= trace["ledger_capacity"] == 8192, "fixed ledger bound")
    ordinals = [event["ordinal"] for event in events]
    require(len(ordinals) == len(set(ordinals)), "duplicate event ordinal")
    for event in events:
        require(isinstance(event["ordinal"], int) and event["ordinal"] > 0 and
                isinstance(event["tid"], int) and 0 < event["tid"] < 2**31 and
                event["operation"] in (1, 2, 3) and event["size"] >= 0,
                "invalid actual allocation identity/size/operation")
    joined, previous_end = [], -1
    for window in windows:
        start, end, tid = (window[key] for key in
                           ("calls_start", "calls_end", "measuring_tid"))
        require(isinstance(start, int) and isinstance(end, int) and
                previous_end <= start <= end and 0 < tid < 2**31,
                "reversed/overlapping window or invalid measuring identity")
        selected = [event for event in events if start < event["ordinal"] <= end]
        require(sorted(event["ordinal"] for event in selected) == list(range(start + 1, end + 1)),
                "every actual measured global ordinal must join exactly once")
        require(tid != pid, "libtest measurement executes on its worker")
        own = [event for event in selected if event["tid"] == tid]
        main = [event for event in selected if event["tid"] == pid]
        other = [event for event in selected if event["tid"] not in (pid, tid)]
        joined.append({**window, "global_calls": end - start,
                       "measuring_calls": len(own), "libtest_main_calls": len(main),
                       "other_worker_calls": len(other), "libtest_main_events": main,
                       "other_worker_events": other})
        previous_end = end
    return joined


def budget_trace(trace, rows):
    joined = exact_windows(trace, range(14))
    for index, row in enumerate(rows):
        for lane in ("native", "retained"):
            window = joined[index * 2 + (lane == "retained")]
            require(window["global_calls"] == row[lane], "stdout/global ordinal count mismatch")
            window.update(fixture=row["fixture"], lane=lane, ceiling=row["ceiling"])
    return joined


def positive_trace(trace, stdout, exit_code):
    require(exit_code == 0 and "scripted_sequence_has_exact_counter_deltas" in stdout and
            "test result: ok. 1 passed; 0 failed;" in stdout,
            "original direct-count and real spawned-worker positive control must pass")
    first, worker = exact_windows(trace, (14, 15))
    require(first["global_calls"] == first["measuring_calls"] == 3, "alloc/zeroed/realloc control")
    require(worker["global_calls"] == 2 and worker["measuring_calls"] == 1 and
            worker["other_worker_calls"] == 1 and worker["libtest_main_calls"] == 0,
            "real spawned routine work remains counted, never filtered")
    return [first, worker]
