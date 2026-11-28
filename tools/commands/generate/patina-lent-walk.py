#!/usr/bin/env python3
"""Generate Patina's lifetime-carrying walk from its checksum-pinned OXC source.

Pass --source the crates/oxc_ast_visit/src/generated/visit.rs from the pinned
OXC checkout. --check regenerates in a temporary directory and checks every
committed output. The Rust tests compare its observable hooks with OXC's walk.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap

REVISION = "fc702c1fa9f0412d06ec6908b58cd395b826cf7f"
SOURCE_PATH = "crates/oxc_ast_visit/src/generated/visit.rs"
SOURCE_SHA256 = "5440cd4c4a2e10effd6b9eafc8d7a283258884b91eae1feef76988ca6760490e"


def tokens(source: str):
    """Yield (text, start, end), skipping comments and all Rust string forms.

    Lifetime apostrophes remain tokens. Character literals, including escaped
    quotes, are skipped. Delimiters in literals/comments cannot balance code.
    """
    at = 0
    while at < len(source):
        if source[at].isspace():
            at += 1
            continue
        if source.startswith("//", at):
            end = source.find("\n", at)
            at = len(source) if end < 0 else end + 1
            continue
        if source.startswith("/*", at):
            level, at = 1, at + 2
            while level:
                if at >= len(source):
                    raise ValueError("unterminated Rust block comment")
                if source.startswith("/*", at):
                    level, at = level + 1, at + 2
                elif source.startswith("*/", at):
                    level, at = level - 1, at + 2
                else:
                    at += 1
            continue
        raw = re.match(r'(?:br|cr|r)(#*)"', source[at:])
        if raw:
            close = '"' + raw.group(1)
            end = source.find(close, at + raw.end())
            if end < 0:
                raise ValueError("unterminated Rust raw string")
            at = end + len(close)
            continue
        if source[at] == '"':
            at += 1
            while at < len(source) and source[at] != '"':
                at += 2 if source[at] == "\\" else 1
            if at >= len(source):
                raise ValueError("unterminated Rust string")
            at += 1
            continue
        if source[at] == "'":
            char = re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", source[at:])
            if char:
                at += char.end()
                continue
        word = re.match(r"[A-Za-z_][A-Za-z_0-9]*", source[at:])
        end = at + (word.end() if word else 1)
        yield source[at:end], at, end
        at = end


def matching(source: str, opening: int) -> int:
    parsed = tokens(source[opening:])
    pairs = {"(": ")", "[": "]", "{": "}"}
    stack = []
    for token, begin, end in parsed:
        if token in pairs:
            stack.append(pairs[token])
        elif token in pairs.values():
            if not stack or token != stack.pop():
                raise ValueError("mismatched Rust delimiters")
            if not stack:
                return opening + end
    raise ValueError("unclosed Rust delimiter")


def edits(source: str, replacements):
    previous = len(source) + 1
    for begin, end, replacement in sorted(replacements, reverse=True):
        if end > previous:
            raise ValueError("overlapping source rewrites")
        source = source[:begin] + replacement + source[end:]
        previous = begin
    return source


def remove_noop_hooks(source: str) -> str:
    source, count = re.subn(r"(?m)^\s*let kind = AstKind::\w+\(visitor\.alloc\(it\)\);\n", "\n", source)
    parsed, replacements = list(tokens(source)), []
    noops = {"enter_node", "leave_node", "enter_scope", "leave_scope"}
    for at in range(len(parsed) - 3):
        if [x[0] for x in parsed[at:at + 2]] != ["visitor", "."]:
            continue
        if parsed[at + 2][0] not in noops or parsed[at + 3][0] != "(":
            continue
        begin = parsed[at][1]
        end = matching(source, parsed[at + 3][1])
        if source[end:end + 1] != ";":
            raise ValueError("hook invocation is no longer a standalone statement")
        end += 1
        line = source.rfind("\n", 0, begin) + 1
        if source[line:begin].strip():
            raise ValueError("hook invocation is no longer on its own line")
        if source[end:end + 1] == "\n":
            end += 1
        replacements.append((line, end, ""))
    return edits(source, replacements)


def direct_walk_calls(source: str) -> str:
    parsed, replacements = list(tokens(source)), []
    for at in range(len(parsed) - 3):
        words = [x[0] for x in parsed[at:at + 4]]
        if words[0:2] != ["visitor", "."] or not words[2].startswith("visit_") or words[3] != "(":
            continue
        # A complete balanced call is required, including when arguments are
        # multiline, macro calls, or contain comments with parentheses.
        matching(source, parsed[at + 3][1])
        replacements.append((parsed[at][1], parsed[at + 2][2], "walk_" + words[2][6:]))
        replacements.append((parsed[at + 3][2], parsed[at + 3][2], "visitor, "))
    return edits(source, replacements)


def make_function(source: str, name: str) -> str:
    source = textwrap.dedent(source).strip() + "\n"
    source = source.replace("pub fn ", "pub(crate) fn ", 1)
    source, bounds = re.subn(r"V: Visit<'a>", "V: LentNodes<'a>", source)
    source, borrows = re.subn(r"\bit: &", "it: &'a ", source)
    if bounds != 1 or borrows != 1:
        raise ValueError(f"unexpected signature: {name}")
    source = direct_walk_calls(remove_noop_hooks(source))
    opening = next(begin for token, begin, end in tokens(source) if token == "{")
    closing = matching(source, opening) - 1
    before = after = ""
    if name in {"walk_jsx_element", "walk_jsx_fragment"}:
        kind = name.removeprefix("walk_")
        before = f"\n    if !visitor.enter_{kind}(it) {{\n        return;\n    }}"
        after = f"    visitor.leave_{kind}(it);\n"
    elif name == "walk_call_expression":
        before = "\n    visitor.call_expression(it);"
    source = source[:closing] + after + source[closing:]
    source = source[:opening + 1] + before + source[opening + 1:]
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path)
    root = Path(__file__).resolve().parents[3]
    parser.add_argument("--output-dir", type=Path,
                        default=root / "crates/vize_patina/src/lent_walk/generated")
    parser.add_argument("--rustfmt", default="rustfmt")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.source is None:
        cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
        candidates = list(cargo_home.glob(f"git/checkouts/oxc-*/{REVISION[:7]}*/{SOURCE_PATH}"))
        if len(candidates) != 1:
            raise SystemExit("pass --source or fetch the pinned OXC dependency with cargo fetch --locked")
        args.source = candidates[0]
    temporary = tempfile.TemporaryDirectory(prefix="vize-lent-walk-")
    committed = args.output_dir
    if args.check:
        args.output_dir = Path(temporary.name)
    raw = args.source.read_bytes()
    if hashlib.sha256(raw).hexdigest() != SOURCE_SHA256:
        raise SystemExit("source hash changed; review OXC traversal before regenerating")
    source = raw.decode()
    start = source.index("pub mod walk {")
    opening = source.index("{", start)
    stop = matching(source, opening)
    walked = source[opening + 1:stop - 1]
    functions = []
    pattern = re.compile(r"(?m)^    (?:#\[inline\]\n    )?pub fn (walk_\w+)")
    for found in pattern.finditer(walked):
        body = next(begin for token, begin, end in tokens(walked[found.end():]) if token == "{") + found.end()
        end = matching(walked, body)
        functions.append((found.group(1), make_function(walked[found.start():end], found.group(1))))
    if len(functions) != 260:
        raise SystemExit(f"expected 260 OXC walk functions, found {len(functions)}")
    combined = "\n".join(body for name, body in functions)
    words = [token for token, _, _ in tokens(combined)]
    if "unsafe" in words or "alloc" in words or "AstKind" in words:
        raise SystemExit("unexpected unchecked lifetime operation remains")
    calls = set(re.findall(r"visitor\.(\w+)\(", combined))
    expected = {"enter_jsx_element", "leave_jsx_element", "enter_jsx_fragment", "leave_jsx_fragment", "call_expression"}
    if calls != expected:
        raise SystemExit(f"unexpected residual visitor hooks: {calls}")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    header = "// Generated from OXC's pinned immutable walk; see mod.rs for provenance.\nuse super::*;\n\n"
    chunks, chunk = [], []
    for name, body in functions:
        if chunk and len((header + "\n".join(text for _, text in chunk) + body).splitlines()) > 450:
            chunks.append(chunk)
            chunk = []
        chunk.append((name, body))
    if chunk:
        chunks.append(chunk)
    for index, chunk in enumerate(chunks, 1):
        (args.output_dir / f"part_{index:02}.rs").write_text(header + "\n".join(body for name, body in chunk))
    declarations = "\n".join(f"mod part_{index:02};\npub(super) use part_{index:02}::*;" for index in range(1, len(chunks) + 1))
    root = f'''//! Private lifetime-carrying immutable AST traversal.
//!
//! Adapted from OXC {REVISION}, `{SOURCE_PATH}`.
//! Original source SHA-256: `{SOURCE_SHA256}`.
//! The walk order and AST matches come from that pinned source. Only the
//! three LentNodes hooks are observable; OXC's other hooks were no-ops for
//! the former Lending visitor. Every node input now borrows for `'a`, so
//! rustc proves that the hooks may retain it for the root's borrow.
//!
//! OXC copyright (c) 2024-present VoidZero Inc. & Contributors; (c) 2023
//! Boshen. The complete MIT license is in the adjacent LICENSE file.

#![allow(
    unused_variables,
    unused_imports,
    clippy::match_same_arms,
    clippy::semicolon_if_nothing_returned,
    clippy::needless_pass_by_ref_mut,
    clippy::trivially_copy_pass_by_ref,
)]

use super::LentNodes;
use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_syntax::scope::ScopeFlags;

{declarations}
'''
    (args.output_dir / "mod.rs").write_text(root)
    license_source = args.source.resolve().parents[4] / "LICENSE"
    (args.output_dir / "LICENSE").write_bytes(license_source.read_bytes())
    subprocess.run([args.rustfmt, "--edition", "2024", str(args.output_dir / "mod.rs")], check=True)
    (args.output_dir / "provenance.json").write_text(json.dumps({
        "revision": REVISION, "source": SOURCE_PATH, "sha256": SOURCE_SHA256,
        "functions": len(functions), "chunks": len(chunks),
        "function_names": [name for name, _ in functions],
        "outputs": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in sorted(args.output_dir.iterdir()) if path.name != "provenance.json"},
    }, indent=2) + "\n")
    if args.check:
        expected = {path.name: path.read_bytes() for path in args.output_dir.iterdir()}
        actual = {path.name: path.read_bytes() for path in committed.iterdir()}
        if expected != actual:
            raise SystemExit("generated Patina walk differs; regenerate from the pinned OXC source")
    print(json.dumps({"output_dir": str(committed), "functions": len(functions),
                      "chunks": len(chunks), "checked": args.check}))


if __name__ == "__main__":
    main()
