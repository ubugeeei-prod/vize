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
GROUP = re.compile(r"(?m)^([ \t]*)((?:pub(?:\([^)]*\))?\s+)?use)\s+vize_l0\s*::\s*\{")

def mask_comments(text):
    out = list(text)
    i = 0
    while i < len(text):
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
                raise SystemExit("Unterminated import comment")
        else:
            i += 1
            continue
        out[i:end] = ["\n" if ch == "\n" else " " for ch in text[i:end]]
        i = end
    return "".join(out)

def grouped_imports(text):
    for match in GROUP.finditer(text):
        depth, start, end = 1, match.end(), match.end()
        while end < len(text):
            if text.startswith("//", end):
                newline = text.find("\n", end)
                end = len(text) if newline == -1 else newline
                continue
            if text.startswith("/*", end):
                comment, end = 1, end + 2
                while end < len(text) and comment:
                    if text.startswith("/*", end):
                        comment += 1
                        end += 2
                    elif text.startswith("*/", end):
                        comment -= 1
                        end += 2
                    else:
                        end += 1
                if comment:
                    raise SystemExit("Unterminated import comment")
                continue
            depth += (text[end] == "{") - (text[end] == "}")
            if depth == 0:
                tail = re.match(r"\s*;", text[end + 1:])
                if not tail:
                    raise SystemExit("Malformed grouped import")
                yield match, text[start:end], end + 1 + tail.end()
                break
            end += 1
        else:
            raise SystemExit("Unterminated grouped import")

def import_items(body):
    depth, start = 0, 0
    for index, ch in enumerate(mask_comments(body)):
        depth += (ch == "{") - (ch == "}")
        if ch == "," and depth == 0:
            item = body[start:index].strip()
            if mask_comments(item).strip():
                yield item + ("\n" if "//" in item else "")
            start = index + 1
    item = body[start:].strip()
    if mask_comments(item).strip():
        yield item + ("\n" if "//" in item else "")

def is_host(item):
    return HOST.match(mask_comments(item).strip()) is not None

def rewrite_grouped_imports(text):
    for match, body, end in reversed(list(grouped_imports(text))):
        items = list(import_items(body))
        host = [item for item in items if is_host(item)]
        if not host:
            continue
        storage = [item for item in items if not is_host(item)]
        indent, prefix = match.groups()
        lines = [f"{indent}{prefix} vize_l0::{{{', '.join(storage)}}};"] if storage else []
        lines += [f"{indent}{prefix} vize_carton::{item};" for item in host]
        text = text[:match.start()] + "\n".join(lines) + text[end:]
    return text

def has_host_import(text):
    if re.search(r"\bvize_l0\s*::\s*(?:r#)?corsa_(?:api_mode|resolver)\b", text):
        return True
    return any(is_host(item) for _, body, _ in grouped_imports(text) for item in import_items(body))

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
            text = text.replace("extern crate vize_l0 as vize_carton;\n", "")
            text = re.sub(r'\bvize_l0::corsa_(api_mode|resolver)\b', r'vize_carton::corsa_\1', text)
            text = rewrite_grouped_imports(text)
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
