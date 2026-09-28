"""Replay the L3 Lean package rename on a fresh main checkout."""

from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
OLD = ROOT / "tests/formal/impeto"
NEW = ROOT / "tests/formal/l3"


def move(source: Path, destination: Path) -> None:
    if source.exists():
        destination.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["git", "mv", str(source.relative_to(ROOT)), str(destination.relative_to(ROOT))],
            cwd=ROOT,
            check=True,
        )


move(OLD, NEW)
move(NEW / "Impeto", NEW / "L3")
move(NEW / "Impeto.lean", NEW / "L3.lean")

tracked = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
for raw in tracked:
    if not raw:
        continue
    relative = raw.decode()
    path = ROOT / relative
    if not path.is_file():
        continue
    active_source = (
        relative.startswith(("crates/", "tests/tooling/", "tests/fuzz/"))
        or relative in ("tests/README.md", ".github/workflows/davinci-lean.yml")
    )
    active_docs = relative.startswith("docs/") and not relative.startswith(
        "docs/davinci/decisions/"
    )
    formal_source = relative.startswith("tests/formal/l3/") and (
        relative.endswith((".lean", "lake-manifest.json", "loop-reference.md"))
    )
    if not active_source and not formal_source and not active_docs:
        continue
    if relative == "tests/tooling/rename-l3-formal.py":
        continue
    try:
        original = path.read_text()
    except (UnicodeError, OSError):
        continue
    changed = original.replace("tests/formal/impeto", "tests/formal/l3")
    changed = changed.replace("formal/impeto", "formal/l3")
    changed = changed.replace('"formal", "impeto"', '"formal", "l3"')
    changed = changed.replace("tests/formal/l3/Impeto/", "tests/formal/l3/L3/")
    changed = changed.replace("formal\\/impeto", "formal\\/l3")
    changed = changed.replace('"Impeto", "', '"L3", "')
    changed = changed.replace("lake exe impetoRef", "lake exe l3Ref")
    if formal_source and relative.endswith(".lean"):
        changed = changed.replace("Impeto", "L3")
        changed = changed.replace("impetoRef", "l3Ref")
        changed = changed.replace("impetoTheorems", "l3Theorems")
        changed = changed.replace("audit_impeto_theorems", "audit_l3_theorems")
        if relative.endswith("lakefile.lean"):
            changed = changed.replace("package impeto", "package l3")
    if relative.endswith("lake-manifest.json"):
        changed = changed.replace('"name": "impeto"', '"name": "l3"')
    if relative.endswith("loop-reference.md"):
        changed = changed.replace("Impeto.", "L3.")
    if active_docs:
        changed = changed.replace("Impeto.", "L3.")
        changed = changed.replace("Impeto/", "L3/")
        changed = changed.replace("audit_impeto_theorems", "audit_l3_theorems")
    if relative in (
        ".github/workflows/davinci-lean.yml",
        "tests/tooling/davinci-lean-workflow.test.ts",
    ):
        changed = changed.replace("impetoRef", "l3Ref")
        changed = changed.replace("impeto-reference", "l3-reference")
        changed = changed.replace("Impeto Lean reference", "L3 Lean reference")
    if relative == "tests/tooling/davinci-lean-workflow.test.ts":
        changed = changed.replace("Impeto\\.", "L3\\.")
        changed = changed.replace("audit_impeto_theorems", "audit_l3_theorems")
    if changed != original:
        path.write_text(changed)
