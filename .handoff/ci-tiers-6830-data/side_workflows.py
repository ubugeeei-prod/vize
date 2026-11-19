"""Move the heaviest path-scoped side workflows off pull_request (#6830)."""
import re, sys

root = sys.argv[1]
note = "  # #6830: off the PR tier; {where}.\n"
drop = {
    "musea-browser": "main runs it per commit and nightly",
    "nuxt-style-build": "main runs it per commit and nightly",
    "nuxt3-module-build": "merge candidates call it and main runs it per commit",
    "parcel-transformer": "main builds every target nightly",
    "v1-supported-matrix": "nightly Full qualification calls it",
    "content-mapper-instance-globals": "its campaign test runs in nightly Full qualification",
    "volar-plugin": "main runs it nightly",
    "canon-ecosystem-producer": "main runs it nightly",
    "jsx-authored-native-modules": "main runs it nightly",
}
nightly = {
    "volar-plugin": "7 4 * * *",
    "canon-ecosystem-producer": "19 4 * * *",
    "jsx-authored-native-modules": "31 4 * * *",
}


def remove_key(text, key):
    lines = text.split("\n")
    out, i, removed = [], 0, False
    in_on = False
    while i < len(lines):
        line = lines[i]
        if re.match(r"^on:\s*$", line):
            in_on = True
        elif in_on and re.match(r"^\S", line):
            in_on = False
        if in_on and re.match(rf"^  {key}:", line):
            removed = True
            i += 1
            while i < len(lines) and (lines[i].startswith("    ") or lines[i].strip() == ""):
                i += 1
            continue
        out.append(line)
        i += 1
    assert removed, key
    return "\n".join(out)


for name, where in drop.items():
    path = f"{root}/.github/workflows/{name}.yml"
    text = open(path).read()
    text = remove_key(text, "pull_request")
    head = re.search(r"^on:\n", text, re.M)
    insert = note.format(where=where)
    if name in nightly:
        assert "\n  schedule:" not in text, name
        insert += f'  schedule:\n    - cron: "{nightly[name]}"\n'
    text = text[: head.end()] + insert + text[head.end():]
    open(path, "w").write(text)
    print("moved", name)

path = f"{root}/.github/workflows/zizmor.yml"
text = open(path).read()
old = "  pull_request: {}\n"
assert old in text
text = text.replace(old, '  # #6830: PRs audit workflows only when they change them.\n  pull_request:\n    paths:\n      - ".github/**"\n')
open(path, "w").write(text)
print("scoped zizmor")
