"""Resolve a CHANGELOG conflict inside `## [Unreleased]` by merging both sides' entries."""
import re, sys

RANK = ["### Added", "### Changed", "### Deprecated", "### Removed", "### Fixed", "### Security"]

def side(region, keep):
    pattern = r'<<<<<<< [^\n]*\n(.*?)=======\n(.*?)>>>>>>> [^\n]*\n'
    return re.sub(pattern, lambda m: m.group(1) if keep == 0 else m.group(2), region, flags=re.S)

def sections(text):
    result, head = {}, None
    for line in text.split("\n"):
        if line.startswith("### "):
            head = line.strip()
            result.setdefault(head, [])
        elif line.startswith("- ") and head:
            result[head].append(line)
        elif line.startswith("  ") and head and result[head]:
            result[head][-1] += "\n" + line
    return result

path = sys.argv[1]
s = open(path).read()
start = s.index("## [Unreleased]\n")
end = s.index("\n## [", start + 1) + 1
region = s[start:end]
merged = {}
for keep in (0, 1):
    for head, bullets in sections(side(region, keep)).items():
        merged.setdefault(head, [])
        for bullet in bullets:
            if bullet not in merged[head]:
                merged[head].append(bullet)
heads = sorted((h for h in merged if merged[h]), key=lambda h: RANK.index(h) if h in RANK else 99)
body = "".join(f"### {h[4:]}\n\n" + "\n".join(merged[h]) + "\n\n" for h in heads)
s = s[:start] + "## [Unreleased]\n\n" + body + s[end:]
assert "<<<<<<<" not in s and ">>>>>>>" not in s
open(path, "w").write(s)
