#!/usr/bin/env python3
"""Accept only the known cross-crate tokenizer re-export false positives.

cargo-semver-checks 0.47.0 cannot resolve items relocated to a dependency and
re-exported from their old paths (upstream issue #355). The paired downstream
consumer fixture must pass against both 0.429.1 and the candidate before
this allowlist is used. Every other SemVer diagnostic remains fatal.
"""

import collections
import pathlib
import re
import sys


def expected_entries() -> dict[str, collections.Counter[str]]:
    enums = ["State", "QuoteType"]
    funcs = ["is_whitespace", "is_tag_start_char", "is_end_of_tag_section"]
    codes = [
        "TAB", "NEWLINE", "FORM_FEED", "CARRIAGE_RETURN", "SPACE",
        "EXCLAMATION_MARK", "DOUBLE_QUOTE", "NUMBER", "AMP", "SINGLE_QUOTE",
        "DASH", "DOT", "SLASH", "ZERO", "NINE", "COLON", "SEMI", "LT",
        "EQ", "GT", "QUESTION_MARK", "AT", "UPPER_A", "UPPER_F", "UPPER_Z",
        "LEFT_SQUARE", "RIGHT_SQUARE", "GRAVE_ACCENT", "LOWER_A", "LOWER_F",
        "LOWER_V", "LOWER_X", "LOWER_Z", "LEFT_BRACE", "RIGHT_BRACE",
    ]

    def both(kind: str, names: list[str]) -> collections.Counter[str]:
        return collections.Counter(
            f"{kind} vize_armature::{prefix}{name}"
            for name in names
            for prefix in ("tokenizer::", "")
        )

    return {
        "enum_missing": both("enum", enums),
        "function_missing": both("function", funcs),
        "module_missing": both("mod", ["char_codes"]),
        "pub_module_level_const_missing": collections.Counter({name: 2 for name in codes}),
        "struct_missing": both("struct", ["Tokenizer"]),
        "trait_missing": both("trait", ["Callbacks"]),
    }


def reported_entries(log: str) -> dict[str, collections.Counter[str]]:
    failures = re.findall(
        r"(?ms)^--- failure ([a-z_]+):[^\n]*\n(.*?)(?=^--- failure |^\s+Summary |\Z)",
        log,
    )
    actual: dict[str, collections.Counter[str]] = {}
    for lint, section in failures:
        if "Failed in:\n" not in section or lint in actual:
            raise ValueError(f"missing evidence or duplicate lint: {lint}")
        entries = section.split("Failed in:\n", 1)[1].split("\n\n", 1)[0]
        parsed: list[str] = []
        for line in entries.splitlines():
            if not line.startswith("  "):
                raise ValueError(f"unexpected diagnostic line: {line}")
            line = line.strip()
            if lint == "pub_module_level_const_missing":
                match = re.fullmatch(r"([A-Z_]+) in file .*/src/tokenizer/char_codes\.rs:\d+", line)
                if not match:
                    raise ValueError(f"unexpected const diagnostic: {line}")
                parsed.append(match.group(1))
            else:
                match = re.fullmatch(r"(.+), previously in file .*/src/tokenizer(?:/types)?\.rs:\d+", line)
                if lint == "module_missing":
                    match = re.fullmatch(r"(.+), previously in file .*/src/tokenizer/char_codes\.rs:\d+", line)
                if not match:
                    raise ValueError(f"unexpected item diagnostic: {line}")
                parsed.append(match.group(1))
        actual[lint] = collections.Counter(parsed)
    return actual


def main() -> int:
    log = pathlib.Path(sys.argv[1]).read_text()
    if "Summary semver requires new major version: 6 major and 0 minor checks failed" not in log:
        print("unexpected SemVer summary", file=sys.stderr)
        return 1
    if "\nerror:" in log or "\npanic" in log:
        print("SemVer command also reported a tool/build error", file=sys.stderr)
        return 1
    try:
        actual = reported_entries(log)
    except ValueError as exc:
        print(exc, file=sys.stderr)
        return 1
    expected = expected_entries()
    if actual != expected:
        for lint in sorted(actual.keys() | expected.keys()):
            unexpected = actual.get(lint, collections.Counter()) - expected.get(lint, collections.Counter())
            missing = expected.get(lint, collections.Counter()) - actual.get(lint, collections.Counter())
            if unexpected or missing:
                print(f"{lint}: unexpected={dict(unexpected)}, missing={dict(missing)}", file=sys.stderr)
        return 1
    print("Only the documented vize_armature tokenizer cross-crate re-export false positives remain.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
