import sys, re
root = sys.argv[1]
for name in sys.argv[2:]:
    text = open(f"{root}/.github/workflows/{name}.yml").read()
    m = re.search(r"^on:\n(.*?)(?=^\S)", text, re.S | re.M)
    block = m.group(1) if m else "(inline on)"
    lines = [l for l in block.splitlines() if not re.match(r'\s+- ["\']', l)]
    print("==", name)
    print("\n".join(lines))
