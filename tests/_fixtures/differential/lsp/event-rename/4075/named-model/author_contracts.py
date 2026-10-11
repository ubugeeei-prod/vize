"""Author immutable packets from literal source coordinates, never editor output."""

import json
from pathlib import Path

ROOT = Path(__file__).parent
FILES = ("Child.vue", "Parent.vue", "Other.vue")
# Explicit authored UTF-16 positions. The first parent site follows an astral glyph.
SITES = (
    ("child", "Child.vue", 3, 21, 29, "saveItem", "nextValue"),
    ("v-model", "Parent.vue", 10, 28, 36, "saveItem", "next-value"),
    ("v-model", "Parent.vue", 11, 17, 26, "save-item", "next-value"),
    ("update", "Parent.vue", 12, 10, 25, "update:saveItem", "update:next-value"),
    ("update", "Parent.vue", 13, 10, 26, "update:save-item", "update:next-value"),
)


def span(line, start, end):
    return {
        "start": {"line": line, "character": start},
        "end": {"line": line, "character": end},
    }


def point_offset(text, point):
    lines = text.splitlines(keepends=True)
    line = lines[point["line"]].rstrip("\r\n")
    prefix = line.encode("utf-16-le")[: point["character"] * 2].decode("utf-16-le")
    return sum(len(value) for value in lines[: point["line"]]) + len(prefix)


def load(prefix, newline):
    return {
        name: (ROOT / f"{prefix}-{name}.txt")
        .read_bytes()
        .decode()
        .replace("\n", newline)
        for name in FILES
    }


def envelope(request_id, result):
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def author_case(origin, newline, new_name):
    sources, goldens = load("input", newline), load("updated", newline)
    references, edits = [], []
    changes = {}
    for _, name, line, start, end, token, replacement in SITES:
        interval = span(line, start, end)
        lo, hi = (point_offset(sources[name], interval[key]) for key in ("start", "end"))
        assert sources[name][lo:hi] == token
        references.append({"uri": "$" + name, "range": interval})
        edits.append({"file": name, "range": interval, "newText": replacement})
        changes.setdefault("$" + name, []).append({"range": interval, "newText": replacement})
    # The complete text golden is independently authored, not made from a reply.
    for name in FILES:
        text = sources[name]
        for edit in reversed([entry for entry in edits if entry["file"] == name]):
            lo, hi = (point_offset(text, edit["range"][key]) for key in ("start", "end"))
            text = text[:lo] + edit["newText"] + text[hi:]
        assert text == goldens[name]
    assert sources["Other.vue"] == goldens["Other.vue"]
    definition = references[0]
    queries = []
    next_id = 2
    for kind, name, line, start, end, token, _ in SITES:
        if kind != origin:
            continue
        interval = span(line, start, end)
        for character in range(start, end):
            params = {
                "textDocument": {"uri": "$" + name},
                "position": {"line": line, "character": character},
            }
            requests, replies = {}, {}
            for method, additional, results in (
                ("prepareRename", {}, [interval, {"range": interval, "placeholder": token}]),
                ("references", {"context": {"includeDeclaration": True}}, [references]),
                ("definition", {}, [definition, [definition]]),
                ("rename", {"newName": new_name}, [{"changes": changes}]),
            ):
                requests[method] = {
                    "jsonrpc": "2.0",
                    "id": next_id,
                    "method": "textDocument/" + method,
                    "params": {**params, **additional},
                }
                replies[method] = [envelope(next_id, result) for result in results]
                next_id += 1
            queries.append({"file": name, "position": params["position"], "requests": requests, "allowedWholeReplies": replies})
    return {
        "origin": origin,
        "newline": newline,
        "newName": new_name,
        "sources": sources,
        "fullReferences": references,
        "fullEdits": edits,
        "queries": queries,
        "selectedApplicationQueryIndex": 0,
        "completeGoldenFiles": goldens,
        "expectedVersion2Diagnostics": {name: [] for name in FILES},
        "expectedIndependentVersion3Diagnostics": {name: [] for name in FILES},
        "negativePreservation": "Other declaration/usages and independent strings stay byte-exact",
    }


def main():
    cases = [
        author_case(origin, newline, name)
        for origin in ("child", "v-model", "update")
        for newline in ("\n", "\r\n")
        for name in ("nextValue", "next-value")
    ]
    assert len(cases) == 12
    assert sum(len(case["queries"]) for case in cases) == 224
    authored = {"schema": "vize-named-model-editor-contracts-v1", "cases": cases}
    (ROOT / "cases.json.txt").write_text(json.dumps(authored, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
