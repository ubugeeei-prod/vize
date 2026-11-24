#!/usr/bin/env python3
"""Readability baseline for the vize repository.

Usage: python3 -I inventory.py <checkout-root> <out-dir>

Writes <out-dir>/files.tsv (one row per in-scope file) and <out-dir>/summary.json.
Every heuristic is a regex or token rule in this file so the numbers can be
re-measured after each lane lands. The heuristics over-approximate on purpose:
they rank files for review, they do not decide edits.
"""
import json
import os
import re
import sys
from collections import Counter, defaultdict

ROOT, OUT = sys.argv[1], sys.argv[2]

LANG = {
    ".rs": "rust", ".ts": "ts", ".tsx": "ts", ".mts": "ts", ".cts": "ts",
    ".js": "js", ".mjs": "js", ".cjs": "js", ".jsx": "js", ".vue": "vue",
    ".pkl": "pkl", ".sh": "shell", ".bash": "shell", ".nix": "nix",
    ".lua": "lua", ".scm": "scheme", ".el": "elisp", ".vim": "vim",
    ".css": "css", ".scss": "css", ".md": "md",
}
EXCLUDED_DIRS = {
    "node_modules", "target", "dist", "fixtures", "_fixtures", "__fixtures__",
    "snapshots", "__snapshots__", "__snapshot__", "expected", "generated",
    "__generated__", "test-fixtures", "testdata", "vendor", "i18n", "corpora",
    "typecheck-corpora", "playwright-report", "coverage", ".vscode-test",
}
EXCLUDED_PREFIXES = (
    "upstream/", ".github/", "tools/benchmarks/results/", "tests/expected/",
    "tests/fuzz/regressions/", "docs/content/ja/", "docs/content/zh-CN/",
    "docs/content/pt-BR/", "docs/content/fr/",
)
EXCLUDED_FILES = {
    "npm/cli/src/types/generated.ts", "npm/native/index.js", "npm/native/index.d.ts",
    "npm/fresco-native/index.js", "npm/fresco-native/index.d.ts",
    "docs/content/rules/all.md",
}


def area(path):
    parts = path.split("/")
    top = parts[0]
    if top in ("crates", "davinci", "npm", "editors", "tools", "tests") and len(parts) > 2:
        return f"{top}/{parts[1]}"
    if top == "docs":
        return "docs/" + (parts[1] if len(parts) > 2 else "(root)")
    return top


def in_scope(path):
    if path in EXCLUDED_FILES or path.startswith(EXCLUDED_PREFIXES):
        return False
    if any(part in EXCLUDED_DIRS for part in path.split("/")[:-1]):
        return False
    if ".min." in path or path.endswith((".d.ts.map",)):
        return False
    return os.path.splitext(path)[1] in LANG


# ---------------------------------------------------------------- comments

def split_c_like(text, lang):
    """Yield (line_no, kind, comment_text) and per-line code text.

    kind: 'line', 'doc' (///, //!, /** */), 'block'. Strings are skipped.
    """
    comments = []
    code_lines = defaultdict(list)
    i, n, line = 0, len(text), 1
    raw_re = re.compile(r'(?:br|r)(#*)"')
    char_re = re.compile(r"'(?:\\.|[^\\'\n])'")
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end == -1 else end
            body = text[i:end]
            if lang == "rust" and (body.startswith("///") and not body.startswith("////") or body.startswith("//!")):
                kind, body = "doc", body[3:]
            elif lang == "pkl" and body.startswith("///"):
                kind, body = "doc", body[3:]
            else:
                kind, body = "line", body[2:]
            comments.append((line, kind, body.strip()))
            i = end
            continue
        if text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = n if end == -1 else end + 2
            body = text[i:end]
            kind = "doc" if body.startswith("/**") and not body.startswith("/**/") or body.startswith("/*!") else "block"
            for k, part in enumerate(body.split("\n")):
                cleaned = re.sub(r"^\s*/?\*+!?/?|\*/\s*$", "", part).strip()
                comments.append((line + k, kind, cleaned))
            line += body.count("\n")
            i = end
            continue
        if lang == "rust":
            m = raw_re.match(text, i)
            if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
                close = '"' + m.group(1)
                end = text.find(close, m.end())
                end = n if end == -1 else end + len(close)
                code_lines[line].append('""')
                line += text.count("\n", i, end)
                i = end
                continue
            if c == "'":
                m = char_re.match(text, i)
                if m:
                    code_lines[line].append("'c'")
                    i = m.end()
                    continue
        quotes = '"' if lang == "rust" else "\"'`"
        if c in quotes:
            q, j = c, i + 1
            while j < n and text[j] != q:
                if text[j] == "\\":
                    j += 1
                j += 1
            lit = text[i:j + 1]
            code_lines[line].append(lit)
            line += lit.count("\n")
            i = j + 1
            continue
        code_lines[line].append(c)
        i += 1
    return comments, {k: "".join(v) for k, v in code_lines.items()}


def split_hash(text, lang):
    marker = {"shell": "#", "nix": "#", "lua": "--", "scheme": ";", "elisp": ";", "vim": '"'}[lang]
    comments, code = [], {}
    for no, raw in enumerate(text.split("\n"), 1):
        s = raw.strip()
        if no == 1 and s.startswith("#!"):
            code[no] = raw
            continue
        if s.startswith(marker):
            comments.append((no, "line", s.lstrip(marker[0]).lstrip(marker).strip()))
        else:
            idx = raw.find(" " + marker + " ")
            if idx > 0 and lang in ("shell", "nix") and raw.count('"') % 2 == 0 and "'" not in raw[:idx]:
                comments.append((no, "trailing", raw[idx + 2:].strip()))
                code[no] = raw[:idx]
            else:
                code[no] = raw
    return comments, code


ISSUE_RE = re.compile(r"(?<![\w&])#\d{3,6}\b|\b(?:issue|PR|pull)\s*#?\d{3,6}\b|\bgh-\d{3,}\b", re.I)
HISTORY_RE = re.compile(
    r"\b(previously|used to|no longer|formerly|originally|historically|"
    r"was (?:changed|renamed|moved|removed|added|introduced)|we now|now (?:uses?|we|it|the|also|only)|"
    r"(?:before|after|since|until) (?:the|this) (?:fix|change|refactor|migration|pr)|"
    r"legacy (?:behaviou?r|path|route)|old (?:code|behaviou?r|path|implementation)|"
    r"regress(?:ed|ion)? (?:in|from|since)|(?:fixed|landed|added|introduced) in v?\d|"
    r"follow-?up|temporar(?:y|ily)|for now\b|stopgap|workaround for #)",
    re.I,
)
BANNER_RE = re.compile(r"^\s*(?:[-=─━*#~_+]{4,}|[-=─━]{2,}\s.*\s[-=─━]{2,})\s*$")
CODEISH_RE = re.compile(
    r"^(?:let |const |var |fn |pub |use |import |export |return\b|if \(|for \(|while \(|"
    r"match |impl |struct |\}\s*$|\{\s*$|[\w.]+\(.*\);\s*$|.*;\s*$|\w+\s*=\s*[^=].*;?$)"
)
KEEP_RE = re.compile(r"^\s*(SAFETY|INVARIANT|Invariant|PANIC|NOTE: unsafe|TODO|FIXME|XXX|HACK|eslint|oxlint|@ts-|prettier|biome|rustfmt|clippy|cspell|#region|#endregion|shellcheck|vite-ignore|istanbul|c8|v8)")
STOP = set("the a an of to in on for is are be this that it its and or with by as at from into we our".split())
WORD_RE = re.compile(r"[A-Za-z][a-z]+|[A-Z]+(?![a-z])|\d+")


def words(text):
    return [w.lower() for w in WORD_RE.findall(text)]


def stem(w):
    for suf in ("ing", "ies", "es", "ed", "s"):
        if w.endswith(suf) and len(w) > len(suf) + 2:
            return w[: -len(suf)]
    return w


def restates(comment, next_code):
    cw = [stem(w) for w in words(comment) if w not in STOP]
    if not cw or len(cw) > 8 or not next_code:
        return False
    kw = {stem(w) for w in words(next_code)}
    hit = sum(1 for w in cw if w in kw)
    return hit / len(cw) >= 0.75


def comment_stats(comments, code, lang):
    stats = Counter()
    by_line = {}
    for no, kind, body in comments:
        by_line.setdefault(no, []).append((kind, body))
    code_nonblank = {k: v for k, v in code.items() if v.strip()}
    sorted_code = sorted(code_nonblank)

    def next_code_line(after):
        lo, hi = 0, len(sorted_code)
        while lo < hi:
            mid = (lo + hi) // 2
            if sorted_code[mid] <= after:
                lo = mid + 1
            else:
                hi = mid
        if lo < len(sorted_code) and sorted_code[lo] - after <= 1:
            return code_nonblank[sorted_code[lo]]
        return None

    # Group consecutive comment lines into blocks.
    blocks, cur = [], []
    for no, kind, body in comments:
        if cur and (no != cur[-1][0] + 1 or kind != cur[-1][1]):
            blocks.append(cur)
            cur = []
        cur.append((no, kind, body))
    if cur:
        blocks.append(cur)
    flagged_lines = set()
    for block in blocks:
        kind = block[0][1]
        text = " ".join(b for _, _, b in block).strip()
        lines = {no for no, _, _ in block}
        stats[f"comment_lines_{'doc' if kind == 'doc' else 'plain'}"] += len(block)
        if not text:
            continue
        if KEEP_RE.match(text):
            stats["keep_marked"] += len(block)
            continue
        flags = []
        # Issue numbers and history narration are counted per line: the line is what an edit removes.
        for no, _, body in block:
            if ISSUE_RE.search(body):
                stats["r_issue"] += 1
                flagged_lines.add(no)
            if HISTORY_RE.search(body):
                stats["r_history"] += 1
                flagged_lines.add(no)
        if all(BANNER_RE.match(b) or not b for _, _, b in block):
            flags.append("banner")
        if kind != "doc" and lang != "md" and sum(1 for _, _, b in block if b and CODEISH_RE.match(b)) >= max(1, len(block) * 0.6) and not re.search(r"\b(e\.g\.|i\.e\.|because|when|so that)\b", text):
            flags.append("commented_code")
        if len(block) <= 2 and restates(text, next_code_line(block[-1][0])):
            flags.append("restates")
        for f in flags:
            stats[f"r_{f}"] += len(block)
        if flags:
            flagged_lines |= lines
    stats["redundant_lines"] = len(flagged_lines)
    return stats


# ---------------------------------------------------------------- code shape

RUST_ITEM = re.compile(r"^\s*(?:#\[|pub(?:\([\w:]+\))?\s+|async\s+|const\s+fn|unsafe\s+|extern\s+)*(?:fn|struct|enum|impl|trait|mod|type|const|static|macro_rules!|union)\b")
RUST_FN = re.compile(r"^\s*(?:pub(?:\([\w:]+\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+\"\w+\"\s+)?fn\s+(\w+)")
TS_FN = re.compile(r"^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s*\*?\s*(\w+)|^\s*(?:export\s+)?const\s+(\w+)\s*(?::[^=]+)?=\s*(?:async\s+)?(?:\([^)]*\)|\w+)\s*(?::[^=]+)?=>")
NUM_RE = re.compile(r"(?<![\w.#$])(?:0x[0-9a-fA-F_]+|\d[\d_]*(?:\.\d+)?)(?:[iu](?:8|16|32|64|128|size)|f32|f64)?(?![\w.])")
TRIVIAL_NUMS = {"0", "1", "2", "-1", "0.0", "1.0", "0.5", "10", "100", "1000", "3", "4", "8", "16", "32", "64"}
STR_RE = re.compile(r'"((?:\\.|[^"\\\n]){4,})"')
REPO_PATH_RE = re.compile(r"""["'`](?:\.\./)*(?:crates|davinci|npm|tools|tests|docs|editors|playground|\.github)/[\w./-]+["'`]""")
HEX_RE = re.compile(r"""["'`][0-9a-f]{40}(?:[0-9a-f]{24})?["'`]""")
ISSUE_IDENT_RE = re.compile(r"\b[A-Za-z_]*?(?:issue|_|-|pr)(\d{4,5})(?:_|-|\b)[A-Za-z_]*", re.I)
CODENAME_RE = re.compile(r"davinci|folio|ricalco|disegno|impeto|sinopia|spolvero|stesura", re.I)


def shape_stats(path, lang, text, code):
    st = Counter()
    lines = text.split("\n")
    st["long_lines_100"] = sum(1 for l in lines if len(l) > 100)
    if lang in ("rust", "ts", "js", "vue"):
        indent_unit = 4 if lang == "rust" else 2
        st["deep_indent"] = sum(
            1 for no, l in code.items() if l.strip() and (len(l) - len(l.lstrip(" "))) >= indent_unit * 7
        )
    # adjacency of items without blank line (rust)
    if lang == "rust":
        prev = ""
        for l in lines:
            s = l.strip()
            if RUST_ITEM.match(l) and prev.strip() == "}" and not s.startswith("#["):
                st["items_no_blank"] += 1
            prev = l
        st["inline_test_mod"] = 1 if re.search(r"#\[cfg\(test\)\]\s*\n\s*mod \w+\s*\{", text) else 0
    # function extents by brace matching on code-only text
    fn_re = RUST_FN if lang == "rust" else TS_FN if lang in ("ts", "js", "vue") else None
    long_fns, longest, nob = [], 0, 0
    if fn_re:
        nos = sorted(code)
        joined = {no: code[no] for no in nos}
        for idx, no in enumerate(nos):
            m = fn_re.match(joined[no])
            if not m:
                continue
            name = next(g for g in m.groups() if g)
            depth, started, end = 0, False, None
            for no2 in nos[idx: idx + 3000]:
                seg = joined[no2]
                if not started and ";" in seg and "{" not in seg and no2 != no:
                    break
                for ch in seg:
                    if ch == "{":
                        depth += 1
                        started = True
                    elif ch == "}":
                        depth -= 1
                if started and depth <= 0:
                    end = no2
                    break
            if end is None:
                continue
            length = end - no + 1
            longest = max(longest, length)
            if length > 80:
                body = lines[no: end - 1]
                blanks = sum(1 for b in body if not b.strip())
                long_fns.append((name, length, blanks))
                if blanks == 0:
                    nob += 1
            if len(name) > 45:
                st["long_fn_names"] += 1
    st["fns_over_80"] = len(long_fns)
    st["fns_over_80_no_blank"] = nob
    st["longest_fn"] = longest
    # hardcoding
    if lang in ("rust", "ts", "js", "vue", "pkl", "shell"):
        in_test = "/tests/" in path or path.startswith("tests/") or re.search(r"(^|[/_.-])tests?([/_.-]|$)", os.path.basename(path)) is not None
        magic = 0
        for no, l in code.items():
            s = l.strip()
            if re.match(r"(?:pub(?:\([\w:]+\))?\s+)?(?:const|static)\s+[A-Z_]", s) or re.match(r"(?:export\s+)?const\s+[A-Z][A-Z0-9_]+\s*=", s):
                continue
            if s.startswith(("#[", "@", "assert", "expect(")):
                continue
            for m in NUM_RE.finditer(re.sub(r'"[^"]*"|\'[^\']*\'|`[^`]*`', '""', l)):
                value = re.sub(r"(?:[iu](?:8|16|32|64|128|size)|f32|f64)$", "", m.group(0)).replace("_", "")
                if value not in TRIVIAL_NUMS:
                    magic += 1
        st["magic_numbers"] = 0 if in_test else magic
        lits = Counter(m.group(1) for l in code.values() for m in STR_RE.finditer(l))
        st["repeated_literals"] = sum(c for s, c in lits.items() if c >= 3)
        st["repo_path_literals"] = sum(len(REPO_PATH_RE.findall(l)) for l in code.values())
        st["hash_literals"] = sum(len(HEX_RE.findall(l)) for l in code.values())
        st["issue_idents"] = sum(len(ISSUE_IDENT_RE.findall(re.sub(r'"[^"]*"', "", l))) for l in code.values())
        st["is_test"] = 1 if in_test else 0
        st["_lits"] = lits
    return st, long_fns


def naming_flags(path, lang):
    base = os.path.basename(path)
    stem_ = base.split(".")[0]
    flags = []
    if lang == "rust" and not re.fullmatch(r"[a-z0-9_]+", stem_):
        flags.append("rust-file-not-snake")
    if lang in ("ts", "js", "vue") and "/" in path:
        if lang == "vue":
            if not re.fullmatch(r"[A-Z][A-Za-z0-9]+|index|App|[a-z0-9-]+", stem_):
                flags.append("vue-file-case")
        elif not re.fullmatch(r"[a-z0-9]+(?:[-.][a-z0-9]+)*|_[a-z0-9-]+|[A-Z][A-Z_]+|index", stem_):
            flags.append("ts-file-not-kebab")
    if re.search(r"[-_]\d{3,6}(?=[-_.]|$)|(?<![a-z])v?\d+\.\d+\.", base):
        flags.append("issue-or-version-in-name")
    if lang == "js" and base.endswith(".mjs"):
        flags.append("retired-mjs")
    if re.fullmatch(r"(?:utils?|helpers?|misc|common|stuff|shared|tmp|temp|new|old|other)", stem_):
        flags.append("generic-name")
    if re.search(r"(?:part|chunk|installment)[-_]?\d+", base, re.I):
        flags.append("numbered-part")
    if CODENAME_RE.search(path):
        flags.append("codename-in-path")
    if len(stem_) > 48:
        flags.append("very-long-name")
    return flags


def main():
    rows = []
    global_lits = defaultdict(set)
    test_name_long = Counter()
    for dp, dn, fn in os.walk(ROOT):
        rel_dir = os.path.relpath(dp, ROOT)
        rel_dir = "" if rel_dir == "." else rel_dir
        dn[:] = [d for d in dn if d != ".git" and not (rel_dir == "" and d == "upstream")]
        for f in fn:
            path = f"{rel_dir}/{f}" if rel_dir else f
            if not in_scope(path):
                continue
            lang = LANG[os.path.splitext(f)[1]]
            try:
                text = open(os.path.join(dp, f), encoding="utf-8").read()
            except (UnicodeDecodeError, OSError):
                continue
            nlines = text.count("\n") + (0 if text.endswith("\n") or not text else 1)
            blank = sum(1 for l in text.split("\n") if not l.strip())
            if lang in ("rust", "ts", "js", "vue", "pkl", "css"):
                src = text
                if lang == "vue":
                    # keep only <script> blocks for comment/code analysis
                    src = "\n".join(
                        l if inside else "" for inside, l in _vue_script_mask(text)
                    )
                comments, code = split_c_like(src, "rust" if lang == "rust" else "pkl" if lang == "pkl" else "ts")
            elif lang == "md":
                comments, code = [], {i: l for i, l in enumerate(text.split("\n"), 1)}
            else:
                comments, code = split_hash(text, lang)
            cs = comment_stats(comments, code, lang) if lang != "md" else Counter()
            ss, long_fns = shape_stats(path, lang, text, code) if lang != "md" else (Counter(), [])
            lits = ss.pop("_lits", Counter())
            if not ss.get("is_test"):
                for s in lits:
                    if len(s) >= 6:
                        global_lits[s].add(path)
            if lang == "rust":
                for m in re.finditer(r"#\[test\]\s*\n\s*(?:async\s+)?fn\s+(\w+)", text):
                    name = m.group(1)
                    if len(name) > 60:
                        test_name_long[area(path)] += 1
                    if re.search(r"\d{4,5}", name):
                        test_name_long[area(path) + "|issue"] += 1
            rows.append({
                "path": path, "area": area(path), "lang": lang, "lines": nlines, "blank": blank,
                **{k: v for k, v in cs.items()}, **{k: v for k, v in ss.items()},
                "naming": naming_flags(path, lang),
                "long_fns": long_fns[:5],
            })
    os.makedirs(OUT, exist_ok=True)
    keys = ["path", "area", "lang", "lines", "blank", "comment_lines_plain", "comment_lines_doc",
            "redundant_lines", "r_issue", "r_history", "r_restates", "r_banner", "r_commented_code",
            "keep_marked", "magic_numbers", "repeated_literals", "repo_path_literals", "hash_literals",
            "issue_idents", "fns_over_80", "fns_over_80_no_blank", "longest_fn", "long_fn_names",
            "items_no_blank", "deep_indent", "long_lines_100", "inline_test_mod", "is_test"]
    with open(os.path.join(OUT, "files.tsv"), "w") as o:
        o.write("\t".join(keys + ["naming"]) + "\n")
        for r in rows:
            o.write("\t".join(str(r.get(k, 0)) for k in keys) + "\t" + ",".join(r["naming"]) + "\n")
    repeated_global = sorted(((len(v), s) for s, v in global_lits.items() if len(v) >= 8), reverse=True)[:80]
    json.dump({"repeated_global_literals": repeated_global, "long_test_names": test_name_long,
               "long_fns": {r["path"]: r["long_fns"] for r in rows if r["long_fns"]}},
              open(os.path.join(OUT, "summary.json"), "w"), indent=1, default=list)


def _vue_script_mask(text):
    inside = False
    for l in text.split("\n"):
        if re.match(r"\s*<script\b", l):
            inside = True
            yield False, l
            continue
        if re.match(r"\s*</script>", l):
            inside = False
        yield inside, l


if __name__ == "__main__":
    main()
