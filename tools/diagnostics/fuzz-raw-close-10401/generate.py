"""Prepare independent counterexamples; the original corpus file stays pinned."""

import argparse
import hashlib
import json
from pathlib import Path

ORIGINAL_SHA256 = "6d668844f745732cdc92bdafa21f72ba5d09d9c14305d4ea9174fb802493dcc8"
PAIRS = [
    ("line-comment", b"\n// x\n", b"\n// >\n", "LineComment"),
    ("block-comment", b"\n/* x */\n", b"\n/* > */\n", "BlockComment"),
    ("double-string", b';\n"x";\n', b';\n">";\n', "StringLiteral"),
    ("single-string", b";\n'x';\n", b";\n'>';\n", "StringLiteral"),
    ("template", b";\n`x`;\n", b";\n`>`;\n", "TemplateLiteral"),
    ("regex", b";\n/[x]/;\n", b";\n/[>]/;\n", "RegExpLiteral"),
    ("binary-token", b";\n1 + 0;\n", b";\n1 > 0;\n", "BinaryExpression"),
]


def digest(value):
    return hashlib.sha256(value).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--scale", action="store_true")
    args = parser.parse_args()
    original = args.input.read_bytes()
    assert len(original) == 23857 and digest(original) == ORIGINAL_SHA256
    assert not args.out.exists(), "fresh counterexample directory required"
    (args.out / "cases").mkdir(parents=True)
    variants = [("original", original, b"", None, "unchanged issue artifact", "original")]
    for name, control, greater, node in PAIRS:
        assert len(control) == len(greater)
        assert control.count(b">") == 0 and greater.count(b">") == 1
        for version, suffix in [("control", control), ("greater", greater)]:
            variants.append((f"{name}-{version}", original, suffix, name,
                             "append-only counterexample, not original artifact", node))
    if args.scale:
        for size in [1024, 2048, 4096, 8192, 16384]:
            prefix = original[:size]
            prefix.decode("utf8")
            for version, suffix in [("control", b"\n// x\n"), ("greater", b"\n// >\n")]:
                variants.append((f"prefix-{size}-{version}", prefix, suffix, f"prefix-{size}",
                                 "prefix-truncated counterexample, not original artifact", "LineComment"))
    cases = []
    for name, base, suffix, pair, source, node in variants:
        data = base + suffix
        (args.out / "cases" / f"{name}.input").write_bytes(data)
        last_gt = data.rfind(b">")
        cases.append({
            "id": name, "pair": pair, "bytes": len(data), "sha256": digest(data),
            "source": source, "base_bytes": len(base), "base_sha256": digest(base),
            "suffix_hex": suffix.hex(), "suffix_utf8": suffix.decode(),
            "suffix_context_when_parsed_alone": node,
            "last_raw_gt": last_gt,
            "raw_lt_positions_before_last_gt": sum(i < last_gt for i, c in enumerate(data) if c == 60),
            "raw_lt_positions_after_last_gt": sum(i >= last_gt for i, c in enumerate(data) if c == 60),
        })
    manifest = {
        "schema_version": 1,
        "baseline_sha": "3a4bf713f85589a50da5d0a7aaf8ae33fdcdc004",
        "candidate_sha": "7bd29589171cdaae96aacca6ee7b213ea6dd347b",
        "original_sha256": ORIGINAL_SHA256, "original_bytes": len(original),
        "candidate_results": "not measured; prepared for exact-source replay",
        "lexical_scope": "suffix syntax is classified alone; concatenated malformed source requires exact-source packet comparison",
        "cases": cases,
    }
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
