"""Exact admission laws for the pinned incomplete-member recovery carrier."""

import copy
import json


def positive(value):
    if type(value) is not int or value <= 0:
        raise ValueError("expected a positive integer")
    return value


def source_counter(sample, name):
    matching = [row for row in sample["micro"] if row["name"] == name]
    if len(matching) != 1:
        raise ValueError("missing or duplicate source counter: " + name)
    row = matching[0]
    return positive(row["calls"]), positive(row["units"])


def validate(profiles):
    if set(profiles) != {"base", "head"}:
        raise ValueError("comparison must have exactly two roles")
    base, head = profiles["base"], profiles["head"]
    signature = ("iterations", "sourceBytes", "sourceHash", "checksum", "rows", "scenario")
    if any(base[key] != head[key] for key in signature):
        raise ValueError("different source or carrier observations")
    if base["scenario"] != "incomplete-member":
        raise ValueError("comparison belongs only to the pinned incomplete-member fixture")
    iterations = positive(base["iterations"])
    source_bytes = positive(base["sourceBytes"])
    counts = {}
    for role, sample in profiles.items():
        if sample.get("instrumented") is not True:
            raise ValueError("allocation laws require an instrumented sample")
        positive(sample["iterations"])
        positive(sample["sourceBytes"])
        original = source_counter(sample, "croquis.oxc.original.bytes")
        recovery = source_counter(sample, "croquis.oxc.recovery.bytes")
        if original != (iterations, iterations * source_bytes):
            raise ValueError("original parsing changed")
        if recovery[1] != recovery[0] * source_bytes:
            raise ValueError("recovery source bytes disagree with actual parse calls")
        if role == "head" and recovery[0] != iterations:
            raise ValueError("candidate must perform exactly one repair parse per input")
        for name in ("calls", "requestedBytes"):
            positive(sample["allocation"][name])
        counts[role] = {"original": original, "recovery": recovery}
    base_recoveries = counts["base"]["recovery"][0]
    if base_recoveries == iterations * 2:
        for name in ("calls", "requestedBytes"):
            if head["allocation"][name] >= base["allocation"][name]:
                raise ValueError("two-to-one requires strictly fewer allocation calls and bytes")
        comparison = "historical-two-to-one-strict-allocation-reduction"
    elif base_recoveries == iterations:
        if counts["base"] != counts["head"] or base["allocation"] != head["allocation"]:
            raise ValueError("one-to-one requires exact source counts and allocation equality")
        comparison = "already-one-to-one-exact-unchanged-control"
    else:
        raise ValueError("unsupported baseline parse count")
    return {"class": comparison, "sourceCounts": counts, "allocation": {
        role: sample["allocation"] for role, sample in profiles.items()
    }}


def fixture(multiplier, calls, allocated):
    iterations, source_bytes = 10, 100
    return {
        "iterations": iterations, "sourceBytes": source_bytes, "sourceHash": 71,
        "checksum": 2570, "rows": 256, "scenario": "incomplete-member", "instrumented": True,
        "micro": [
            {"name": "croquis.oxc.original.bytes", "calls": iterations, "units": iterations * source_bytes},
            {"name": "croquis.oxc.recovery.bytes", "calls": iterations * multiplier, "units": iterations * multiplier * source_bytes},
        ],
        "allocation": {"calls": calls, "requestedBytes": allocated},
    }


def self_test():
    head = fixture(1, 70, 8000)
    two = {"base": fixture(2, 100, 10000), "head": head}
    one = {"base": copy.deepcopy(head), "head": head}
    assert validate(two)["class"].startswith("historical-two-to-one")
    assert validate(one)["class"].startswith("already-one-to-one")
    rejected = []
    cases = []
    def case(name, source, mutate):
        value = copy.deepcopy(source)
        mutate(value)
        cases.append((name, value))
    case("missing-source-counter", one, lambda value: value["head"]["micro"].pop())
    case("duplicate-source-counter", one, lambda value: value["head"]["micro"].append(copy.deepcopy(value["head"]["micro"][-1])))
    case("zero-iterations", one, lambda value: [value[role].__setitem__("iterations", 0) for role in ("base", "head")])
    case("unsupported-base-count", two, lambda value: value.__setitem__("base", fixture(3, 100, 10000)))
    case("head-two-repairs", two, lambda value: value.__setitem__("head", fixture(2, 70, 8000)))
    case("recovery-byte-mismatch", one, lambda value: value["head"]["micro"][-1].__setitem__("units", 999))
    case("original-call-mismatch", one, lambda value: value["head"]["micro"][0].__setitem__("calls", 9))
    case("same-class-call-increase", one, lambda value: value["head"]["allocation"].__setitem__("calls", 71))
    case("same-class-byte-increase", one, lambda value: value["head"]["allocation"].__setitem__("requestedBytes", 8001))
    case("same-class-unrelated-decrease", one, lambda value: value["head"]["allocation"].__setitem__("requestedBytes", 7999))
    case("two-to-one-no-byte-win", two, lambda value: value["head"]["allocation"].__setitem__("requestedBytes", 10000))
    case("two-to-one-no-call-win", two, lambda value: value["head"]["allocation"].__setitem__("calls", 100))
    case("source-identity-mismatch", one, lambda value: value["head"].__setitem__("sourceHash", 72))
    case("unowned-scenario", one, lambda value: [value[role].__setitem__("scenario", "valid") for role in ("base", "head")])
    case("non-instrumented-sample", one, lambda value: value["head"].__setitem__("instrumented", False))
    case("boolean-iterations", one, lambda value: value["head"].__setitem__("iterations", True))
    case("unexpected-role", one, lambda value: value.__setitem__("extra", copy.deepcopy(value["head"])))
    for name, value in cases:
        try:
            validate(value)
        except (KeyError, TypeError, ValueError):
            rejected.append(name)
        else:
            raise AssertionError("invalid comparison admitted: " + name)
    print("recovery-class-controls=" + json.dumps({"positive": 2, "rejected": rejected}))


if __name__ == "__main__":
    self_test()
