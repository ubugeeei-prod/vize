import re, subprocess, sys
root = sys.argv[1]
files = subprocess.run(["git", "-C", root, "grep", "-l", "fn load() -> Value", "--", "crates/vize_canon/tests/empty_vue_star_exports"], capture_output=True, text=True).stdout.split()
for f in files:
    src = open(f"{root}/{f}").read()
    m = re.search(r"\n((?:pub\(super\) )?fn load\(\) -> Value \{.*?\n\})", src, re.S)
    print("==", f)
    print(m.group(1)[:1500] if m else "??")
    # who calls load()
    print("  callers:", len(re.findall(r"\bload\(\)", src)) - 1)
