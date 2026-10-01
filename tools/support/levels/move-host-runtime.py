#!/usr/bin/env python3
"""Replay the #6834 host Corsa owner move on current main."""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
FILES = (
    "corsa_api_mode.rs",
    "corsa_resolver.rs",
    "corsa_resolver/normalize_tests.rs",
    "corsa_resolver/tests.rs",
    "corsa_resolver/tests/package_runtime.rs",
)
CONSUMERS = ("vize", "vize_canon", "vize_maestro", "vize_patina")

def paths(relative):
    return ROOT / "davinci/vize_l0/src" / relative, ROOT / "crates/vize_carton/src" / relative

def update(path, change):
    p = ROOT / path
    before = p.read_text(encoding="utf-8")
    after = change(before)
    if before != after:
        p.write_text(after, encoding="utf-8")

HOST = re.compile(r"^(?:r#)?corsa_(?:api_mode|resolver)\b")
GROUP = re.compile(r"(?<!\w)([ \t]*)((?:pub(?:\([^)]*\))?\s+)?use)\s+vize_l0\s*::\s*\{")

RAW_STRING = re.compile(r'(?:br|cr|r)(#*)"')
CHAR = re.compile(r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[^\n])|[^\\'\n])'")
LEX_START = re.compile(r'//|/\*|(?<!\w)(?:br|cr|r)#*"|["\x27]')
DIRECT = re.compile(r"\bvize_l0(?=\s*::\s*(?:r#)?corsa_(?:api_mode|resolver)\b)")

def lexical_view(text):
    """Mask Rust comments/literals without changing offsets or line boundaries."""
    out, i = list(text), 0
    while i < len(text):
        token = LEX_START.search(text, i)
        if token is None:
            break
        i = token.start()
        raw = RAW_STRING.match(text, i)
        char = CHAR.match(text, i)
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = len(text) if end == -1 else end
        elif text.startswith("/*", i):
            end, depth = i + 2, 1
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            if depth:
                raise SystemExit("Unterminated Rust comment")
        elif raw:
            closing = '"' + raw.group(1)
            end = text.find(closing, raw.end())
            if end == -1:
                raise SystemExit("Unterminated raw string")
            end += len(closing)
        elif text[i] == '"':
            end = i + 1
            while end < len(text) and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            if end >= len(text):
                raise SystemExit("Unterminated string")
            end += 1
        elif char:
            end = char.end()
        else:
            i += 1
            continue
        out[i:end] = ["\n" if ch == "\n" else " " for ch in text[i:end]]
        i = end
    return "".join(out)

def import_attributes(text, view, start):
    """Include every preceding outer attribute in a split import's prefix."""
    first, cursor = start, start
    while True:
        cursor -= 1
        while cursor >= 0 and view[cursor].isspace():
            cursor -= 1
        if cursor < 0 or view[cursor] != "]":
            break
        depth, opening = 1, cursor - 1
        while opening >= 0 and depth:
            depth += (view[opening] == "]") - (view[opening] == "[")
            opening -= 1
        while opening >= 0 and view[opening].isspace():
            opening -= 1
        if depth or opening < 0 or view[opening] != "#":
            break
        first = opening
        while first > 0 and text[first - 1] in " \t":
            first -= 1
        cursor = first
    return first, text[first:start]

def grouped_imports(text):
    view = lexical_view(text)
    for match in GROUP.finditer(view):
        depth, start, end = 1, match.end(), match.end()
        while end < len(view):
            depth += (view[end] == "{") - (view[end] == "}")
            if depth == 0:
                tail = re.match(r"\s*;", view[end + 1:])
                if not tail:
                    raise SystemExit("Malformed grouped import")
                first, attributes = import_attributes(text, view, match.start())
                yield match, text[start:end], end + 1 + tail.end(), first, attributes
                break
            end += 1
        else:
            raise SystemExit("Unterminated grouped import")

def import_items(body):
    depth, start = 0, 0
    for index, ch in enumerate(lexical_view(body)):
        depth += (ch == "{") - (ch == "}")
        if ch == "," and depth == 0:
            item = body[start:index].strip()
            if lexical_view(item).strip():
                yield item + ("\n" if "//" in item else "")
            start = index + 1
    item = body[start:].strip()
    if lexical_view(item).strip():
        yield item + ("\n" if "//" in item else "")

def is_host(item):
    return HOST.match(lexical_view(item).strip()) is not None

def rewrite_grouped_imports(text):
    for match, body, end, first, attributes in reversed(list(grouped_imports(text))):
        items = list(import_items(body))
        host = [item for item in items if is_host(item)]
        if not host:
            continue
        storage = [item for item in items if not is_host(item)]
        indent = match.group(1)
        prefix = text[match.start(2):match.end(2)]
        lines = [f"{indent}{prefix} vize_l0::{{{', '.join(storage)}}};"] if storage else []
        lines += [f"{indent}{prefix} vize_carton::{item};" for item in host]
        text = text[:first] + "\n".join(attributes + line for line in lines) + text[end:]
    return text

def rewrite_host_imports(text):
    """Rewrite real qualified host paths and grouped uses, never literal data."""
    for match in reversed(list(DIRECT.finditer(lexical_view(text)))):
        text = text[:match.start()] + "vize_carton" + text[match.end():]
    return rewrite_grouped_imports(text)

def has_host_import(text):
    if DIRECT.search(lexical_view(text)):
        return True
    return any(is_host(item) for _, body, _, _, _ in grouped_imports(text) for item in import_items(body))

def integrate():
    update("crates/vize_carton/src/corsa_resolver/tests.rs", lambda s: s.replace("error.to_string()", 'crate::cstr!("{error}")'))
    update("davinci/vize_l0/src/lib.rs", lambda s: re.sub(
        r'#\[cfg\(not\(target_arch = "wasm32"\)\)\]\npub mod corsa_(?:api_mode|resolver);\n', "", s))
    update("crates/vize_carton/src/lib.rs", lambda s: s if "pub mod corsa_resolver;" in s else
        s.replace("//! Legacy compatibility facade for the shared foundation.",
                  "//! Legacy storage compatibility and host-runtime integration.")
        + '\n#[cfg(not(target_arch = "wasm32"))]\npub mod corsa_api_mode;\n'
        + '#[cfg(not(target_arch = "wasm32"))]\n'
        + '#[expect(\n    clippy::disallowed_types,\n    reason = "legacy host discovery retains its environment and path collections"\n)]\n'
        + 'pub mod corsa_resolver;\n')
    update("davinci/vize_l0/Cargo.toml", lambda s: s.replace(
        '[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\nwhich = { workspace = true }\n\n', ""))
    update("crates/vize_carton/Cargo.toml", lambda s: s if "which = { workspace = true }" in s else
        s.replace("[features]\n",
                  '[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\n'
                  'which = { workspace = true }\nserde_json.workspace = true\n\n[features]\n'))
    for name in CONSUMERS:
        manifest = Path("crates") / name / "Cargo.toml"
        update(manifest, lambda s: s if re.search(r'^vize_carton\s*[.=]', s, re.M) else
               s.replace("[dependencies]\n", "[dependencies]\nvize_carton.workspace = true\n"))
        for source in (ROOT / "crates" / name).rglob("*.rs"):
            text = source.read_text(encoding="utf-8")
            # Remove Canon's private storage alias, which otherwise shadows the real host owner.
            for alias in reversed(list(re.finditer(r"(?m)^extern crate vize_l0 as vize_carton;\n", lexical_view(text)))):
                text = text[:alias.start()] + text[alias.end():]
            text = rewrite_host_imports(text)
            if name == "vize" and source.name == "check_cli.rs":
                text = text.replace('};\n\nuse vize_l0::{cstr, path::canonicalize_non_verbatim};\n\nuse vize_carton::corsa_resolver::platform_suffix;',
                                    '};\nuse vize_l0::{cstr, path::canonicalize_non_verbatim};\nuse vize_carton::corsa_resolver::platform_suffix;')
            if text != source.read_text(encoding="utf-8"):
                source.write_text(text, encoding="utf-8")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("moves", "integrate", "check"))
    phase = parser.parse_args().phase
    locations = [paths(relative) for relative in FILES]
    for old, new in locations:
        if old.exists() == new.exists():
            raise SystemExit("Missing source or move collision: " + str(old.relative_to(ROOT)))
        if phase != "moves" and old.exists():
            raise SystemExit("Run the move-only phase first")
    if phase == "moves":
        for old, new in locations:
            if old.exists():
                new.parent.mkdir(parents=True, exist_ok=True)
                subprocess.run(["git", "mv", str(old), str(new)], cwd=ROOT, check=True)
    elif phase == "integrate":
        integrate()
    else:
        for name in CONSUMERS:
            for source in (ROOT / "crates" / name).rglob("*.rs"):
                if has_host_import(source.read_text(encoding="utf-8")):
                    raise SystemExit("Unmigrated host import: " + str(source.relative_to(ROOT)))
        print("Host runtime ownership is integrated")

if __name__ == "__main__":
    main()
