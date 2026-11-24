#!/usr/bin/env python3
"""Insert one blank line between a closing `}` and the next item at the same
indentation (style guide §1.5 rule 1). Usage: item_spacing.py [--dry] <file>..."""
import re, sys
ITEM = re.compile(r"(?:#\[|///|//!|(?:pub(?:\([\w:\s]+\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+\"\w+\"\s+)*(?:fn|struct|enum|impl|trait|mod|type|const|static|macro_rules!|union|use)\b)")
dry = "--dry" in sys.argv
total = 0
for path in [a for a in sys.argv[1:] if a != "--dry"]:
    lines = open(path, encoding="utf-8").read().split("\n")
    out, n = [], 0
    for i, line in enumerate(lines):
        if i > 0:
            prev = lines[i - 1]
            indent = len(prev) - len(prev.lstrip(" "))
            if prev.strip() == "}" and line.strip() and len(line) - len(line.lstrip(" ")) == indent and ITEM.match(line.strip()):
                out.append("")
                n += 1
                if dry:
                    print(f"{path}:{i + 1}: {line.strip()[:80]}")
        out.append(line)
    total += n
    if n and not dry:
        open(path, "w", encoding="utf-8").write("\n".join(out))
print(f"{total} blank lines {'would be ' if dry else ''}inserted", file=sys.stderr)
