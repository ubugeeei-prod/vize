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
            # Split grouped imports so only host APIs come from Carton.
            def imports(match):
                indent, items = match.groups()
                host = []
                def extract(m):
                    host.append(m.group(1))
                    return ""
                items = re.sub(r'(corsa_(?:api_mode|resolver)::(?:\{[^}]*\}|[A-Za-z_][A-Za-z_0-9]*)),?\s*',
                               extract, items)
                if not host:
                    return match.group(0)
                items = items.strip().strip(",").strip()
                storage = f"{indent}use vize_l0::{{{items}}};" if items else ""
                return storage + "".join(f"\n{indent}use vize_carton::{item};" for item in host)
            text = re.sub(r'(?m)^([ \t]*)use vize_l0::\{((?:[^{}]|\{[^{}]*\})*)\};', imports, text)
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
                if re.search(r'\bvize_l0::corsa_(api_mode|resolver)\b', source.read_text(encoding="utf-8")):
                    raise SystemExit("Unmigrated host import: " + str(source.relative_to(ROOT)))
        print("Host runtime ownership is integrated")

if __name__ == "__main__":
    main()
