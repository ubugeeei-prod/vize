"""Verify a retained capsule, then derive a bounded, explicitly scoped summary."""
import hashlib
import json
from pathlib import Path
import re
import sys

def sha(body):
    return hashlib.sha256(body).hexdigest()

def summarize(root):
    root = Path(root)
    manifest_bytes = (root / "manifest.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    assert not manifest["errors"] and not manifest["requiredMissing"]
    files = {}
    identities = {}
    for entry in manifest["selected"]:
        path = entry["path"]
        assert ".." not in Path(path).parts and not Path(path).is_absolute()
        body = (root / "cold-scale" / path).read_bytes()
        assert len(body) == entry["bytes"] and sha(body) == entry["sha256"], path
        files[path] = body
        identities[path] = {"bytes": len(body), "sha256": sha(body)}
    result = {
        "schemaVersion": 1, "sourceRun": manifest["sourceRun"],
        "sourceArtifact": manifest["sourceArtifact"],
        "sourceArtifactApi": manifest["sourceArtifactApi"],
        "downloadedTarSha256": manifest["downloadedTarSha256"],
        "manifestSha256": sha(manifest_bytes), "verifiedMembers": len(files),
        "verifiedBytes": manifest["totalBytes"],
        "builds": [], "measurements": [], "profiles": [], "pmu": [], "parseErrors": [],
        "newBuild": False, "newCapture": False, "newCheckExecution": False,
        "coldQualified": False, "hardwareCounterAcceptance": None,
    }
    for path, body in files.items():
        if path.startswith("builds/") and path.endswith(".json"):
            value = json.loads(body)
            keys = ("role", "sourceSha", "treeSha", "binary", "binarySha256", "binaryBytes",
                    "allocation", "targetDir", "targetStartedEmpty", "profile",
                    "compilerEnvironment", "qualified", "reason", "base", "head")
            row = {key: value[key] for key in keys if key in value}
            for side in ("base", "head"):
                if isinstance(row.get(side), dict):
                    row[side] = {k:v for k,v in row[side].items() if k in ("role", "sourceSha", "binarySha256", "allocation", "targetStartedEmpty", "qualified")}
            row["cargoArtifact"] = value.get("cargoArtifact")
            result["builds"].append({"path": path, "identity": identities[path], "receipt": row})
        if re.fullmatch(r"profile-(?:base|head)/profile.json", path):
            value = json.loads(body)
            spans = value.get("spans", [])
            chosen = {}
            for sort in (lambda v: (v.get("alloc") or {}).get("self_calls", 0),
                         lambda v: (v.get("alloc") or {}).get("bytes", 0),
                         lambda v: (v.get("wall_ns") or {}).get("self", 0),
                         lambda v: v.get("count", 0)):
                for span in sorted(spans, key=sort, reverse=True)[:8]:
                    chosen[span["key"]] = span
            counters = value.get("counters", [])
            selected = [v for v in counters if re.search(
                r"route|workspace|hash|alloc|source|function|call|read|stat|realpath|parse|analyze|type.query",
                v.get("key", ""), re.I)]
            allocation = value.get("allocation")
            peak = next((allocation.get(key) for key in ("peak_live_bytes", "peakLiveBytes", "peak_live")
                         if isinstance(allocation, dict) and key in allocation), None)
            result["profiles"].append({
                "path": path, "identity": identities[path],
                "tool": value.get("tool"), "version": value.get("tool_version"),
                "allocation": allocation, "peakLiveBytes": peak,
                "truncation": value.get("truncation"),
                "spans": list(chosen.values()), "spanCount": len(spans),
                "counters": selected[:80], "selectedCounterCount": len(selected),
                "counterSummaryTruncated": len(selected) > 80,
                "hashCountersPresent": any("hash" in v.get("key", "").lower() for v in counters),
                "scope": "profiling-build requested allocation volume and instrumented spans; span entries are not universal physical calls; wall spans are not normal latency",
            })
        if path.endswith("/measurement.json"):
            value = json.loads(body)
            stdout = files.get(path.rsplit("/", 1)[0] + "/stdout.txt")
            packet = None
            if stdout:
                try:
                    packet = json.loads(stdout)
                except ValueError:
                    pass
            packet_counts = None
            if isinstance(packet, dict) and isinstance(packet.get("files"), list):
                rows = packet["files"]
                names = [row.get("file") for row in rows if isinstance(row, dict)]
                packet_counts = {"arrayLength": len(rows), "fileCount": packet.get("fileCount"),
                                 "uniqueNames": len(set(names)),
                                 "vue": sum(isinstance(name, str) and name.endswith(".vue") for name in names),
                                 "typescript": sum(isinstance(name, str) and re.search(r"\.[cm]?tsx?$", name) is not None for name in names)}
            corpus_path = path.rsplit("/", 1)[0] + "/corpus.json"
            corpus = None
            if corpus_path in files:
                raw_corpus = json.loads(files[corpus_path])
                rows = raw_corpus.get("entries", raw_corpus.get("files", []))
                assert isinstance(rows, list), "corpus entries unavailable"
                names = [row.get("path", row.get("file", row.get("relativePath"))) for row in rows if isinstance(row, dict)]
                assert all(isinstance(name, str) for name in names), "corpus path unavailable"
                computed_bytes = sum(row["bytes"] for row in rows)
                computed_hash = sha(json.dumps(rows, ensure_ascii=False, separators=(",", ":")).encode())
                assert computed_bytes == raw_corpus["sourceBytes"]
                assert computed_hash == raw_corpus["corpusSha256"]
                assert len(set(names)) == len(names)
                corpus = {"identity": identities[corpus_path], "arrayLength": len(rows),
                          "computedSourceBytes": computed_bytes, "computedCorpusSha256": computed_hash,
                          "uniqueNames": len(set(names)),
                          "vue": sum(isinstance(name, str) and name.endswith(".vue") for name in names),
                          "typescript": sum(isinstance(name, str) and re.search(r"\.[cm]?tsx?$", name) is not None for name in names),
                          "rootScalars": {k:v for k,v in raw_corpus.items() if isinstance(v,(int,float,str,bool,type(None)))}}
            stderr = files.get(path.rsplit("/", 1)[0] + "/stderr.txt", b"").decode()
            rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", stderr)
            keys = ("tool", "version", "files", "sfcFiles", "sourceBytes", "corpusSha256",
                    "wallMs", "status", "signal", "executableSha256", "backendSha256",
                    "args", "cold", "meetsColdTarget", "correctness")
            values = {key: value[key] for key in keys if key in value}
            correctness = values.get("correctness")
            if isinstance(correctness, dict) and "diagnostics" in correctness:
                correctness = dict(correctness)
                diagnostics = correctness.pop("diagnostics")
                correctness["diagnosticArrayLength"] = len(diagnostics) if isinstance(diagnostics, list) else None
                values["correctness"] = correctness
            result["measurements"].append({
                "path": path, "identity": identities[path], "receipt": values,
                "actualOutputCounts": packet_counts, "actualCorpusCounts": corpus,
                "maxProcessOrWaitedChildRssKib": int(rss[1]) if rss else None,
                "rssScope": "GNU time maximum process/waited child, not simultaneous process-tree sum",
            })
        if path.endswith("/calibration.json"):
            value = json.loads(body)
            result["pmu"].append({"path": path, "identity": identities[path], "qualified": value.get("qualified"),
                "groups": value.get("groups"), "reason": value.get("reason"),
                "hardwareCounterAcceptance": None if not value.get("qualified") else "inspect individual counter receipts"})
        if re.fullmatch(r"stat-(?:core|cache)-(?:base|head|mapper)/.*/receipt.json", path):
            value = json.loads(body)
            result["pmu"].append({"path": path, "identity": identities[path],
                "qualified": value.get("qualified"), "collection": value.get("collection"),
                "metric": value.get("metric"), "capability": value.get("capability"),
                "calibrationQualified": value.get("calibrationQualified"),
                "acceptedCounters": value.get("counters") if value.get("qualified") else None})
    return result

if __name__ == "__main__":
    source, destination = sys.argv[1:]
    result = summarize(source)
    body = (json.dumps(result, separators=(",", ":")) + "\n").encode()
    assert len(body) <= 100 * 1024, "bounded summary exceeded 100KiB; full custody stays remote"
    Path(destination).write_bytes(body)
    print(json.dumps({"summaryBytes": len(body), "verifiedMembers": result["verifiedMembers"],
                      "sourceRun": result["sourceRun"], "sourceArtifact": result["sourceArtifact"]}))
