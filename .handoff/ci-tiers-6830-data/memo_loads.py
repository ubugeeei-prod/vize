"""Memoize every argument-free fixture record loader in the empty_vue_star_exports test.

Each loader reads one committed JSON record, authenticates it and returns it.
Successor layers call predecessor loaders once per projected file, so an
uncached chain re-reads and re-authenticates the same records combinatorially.
"""
import re, subprocess, sys

root = sys.argv[1]
files = subprocess.run(
    ["git", "-C", root, "grep", "-l", "fn load() -> Value {", "--", "crates/vize_canon/tests/empty_vue_star_exports"],
    capture_output=True, text=True, check=True,
).stdout.split()
for f in files:
    path = f"{root}/{f}"
    src = open(path).read()
    pattern = re.compile(r"^((?:pub\(super\) )?)fn load\(\) -> Value \{\n", re.M)
    matches = pattern.findall(src)
    assert len(matches) == 1, (f, matches)
    assert "fn load_uncached" not in src and "LOADED" not in src, f
    src = pattern.sub(
        lambda m: (
            f"/// Every caller reads the same authenticated record; authenticate it once.\n"
            f"{m.group(1)}fn load() -> Value {{\n"
            f"    static LOADED: std::sync::OnceLock<Value> = std::sync::OnceLock::new();\n"
            f"    LOADED.get_or_init(load_uncached).clone()\n"
            f"}}\n\n"
            f"fn load_uncached() -> Value {{\n"
        ),
        src,
    )
    open(path, "w").write(src)
    print("memoized", f)
