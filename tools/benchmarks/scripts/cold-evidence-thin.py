"""Copy only bounded textual cold-scale evidence; never execute captured content."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import sys
import tarfile

PER_FILE = 8 * 1024 * 1024
TOTAL = 32 * 1024 * 1024
ROOT_FILES = {"baseline-selection.json", "base.sha", "head.sha", "package.json",
              "package-lock.json", "addr2line-version.txt"}
MEASURES = {"measurement.json", "corpus.json", "stdout.txt", "stderr.txt", "profile.json"}
REQUIRED = {
    "base.sha", "head.sha", "builds/normal-base.json", "builds/normal-head.json",
    "builds/normal-pair.json", "builds/profile-base.json", "builds/profile-head.json",
    "builds/profile-pair.json", "profile-base/profile.json", "profile-head/profile.json",
    "pmu-controls/calibration.json",
}


def selected(name):
    clean = name[2:] if name.startswith("./") else name
    path = PurePosixPath(clean)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        return None
    if path.parts[0] != "cold-scale" or len(path.parts) < 2:
        return None
    parts = path.parts[1:]
    if any(part in {"project", "node_modules", "tmp", "xdg-cache", "binaries"}
           for part in parts):
        return None
    relative = str(PurePosixPath(*parts))
    if len(parts) == 1:
        return relative if relative in ROOT_FILES else None
    if parts[0] == "builds" and len(parts) == 2 and path.suffix == ".json":
        return relative
    stage = r"(?:base|head)-[123]|mapper|vue-tsc|control-(?:cli|mapper|vue-tsc)|profile-(?:base|head)"
    if re.fullmatch(stage, parts[0]) and len(parts) == 2 and parts[1] in MEASURES:
        return relative
    observers = r"pmu-controls|stat-(?:core|cache)-(?:base|head|mapper)|record-(?:head|mapper)"
    if re.fullmatch(observers, parts[0]) and path.suffix in {".json", ".txt", ".csv", ".log"}:
        return relative
    return None


def collect(stream, write, required=REQUIRED):
    manifest = {"selected": [], "errors": [], "totalBytes": 0, "requiredMissing": []}
    seen = set()
    for member in stream:
        path = selected(member.name)
        if path is None:
            continue
        if not member.isfile() or path in seen or member.size > PER_FILE:
            manifest["errors"].append({"path": path, "reason": "nonregular, duplicate, or oversized"})
            continue
        seen.add(path)
        if manifest["totalBytes"] + member.size > TOTAL:
            manifest["errors"].append({"path": path, "reason": "total metadata limit"})
            continue
        reader = stream.extractfile(member)
        if reader is None:
            manifest["errors"].append({"path": path, "reason": "missing regular body"})
            continue
        body = reader.read(PER_FILE + 1)
        if len(body) != member.size:
            manifest["errors"].append({"path": path, "reason": "truncated body"})
            continue
        body.decode("utf8")
        write(path, body)
        manifest["totalBytes"] += len(body)
        manifest["selected"].append({"path": path, "bytes": len(body),
                                     "sha256": hashlib.sha256(body).hexdigest()})
    copied = {entry["path"] for entry in manifest["selected"]}
    manifest["requiredMissing"] = sorted(required - copied)
    return manifest


def main():
    archive, out_name, run_id, artifact_id, api_file = sys.argv[1:]
    assert re.fullmatch(r"[0-9]+", run_id) and re.fullmatch(r"[0-9]+", artifact_id)
    api = json.loads(Path(api_file).read_text())
    assert str(api["id"]) == artifact_id and str(api["workflow_run"]["id"]) == run_id
    assert not api["expired"] and api["size_in_bytes"] <= 300 * 1024 * 1024
    out = Path(out_name)
    out.mkdir(exist_ok=False)
    def write(path, body):
        target = out / "cold-scale" / path
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as destination:
            destination.write(body)
    with tarfile.open(archive, "r|gz") as stream:
        manifest = collect(stream, write)
    digest = hashlib.sha256()
    with open(archive, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    manifest.update({
        "schemaVersion": 1, "sourceRun": run_id, "sourceArtifact": artifact_id,
        "sourceArtifactApi": api, "downloadedTarSha256": digest.hexdigest(),
        "driverSource": os.environ.get("GITHUB_SHA"),
        "analysisPerformed": False, "newBuild": False, "newCapture": False,
        "newCheckExecution": False, "coldQualified": False,
        "scope": "verbatim metadata copy only; declared ZIP digest is from API, tar digest independently computed",
    })
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    assert not manifest["errors"] and not manifest["requiredMissing"], manifest
    print(json.dumps({"selectedFiles": len(manifest["selected"]), "bytes": manifest["totalBytes"],
                      "sourceRun": run_id, "sourceArtifact": artifact_id, "newExecution": False}))



def self_test():
    import io
    assert selected("cold-scale/profile-base/profile.json") == "profile-base/profile.json"
    assert selected("./cold-scale/base-3/corpus.json") == "base-3/corpus.json"
    for name in ("cold-scale/binaries/normal-head/vize", "cold-scale/record-head/perf.data",
                 "cold-scale/record-head/project/secret.json", "../cold-scale/base.sha",
                 "/cold-scale/base.sha", "cold-scale/record-head/tmp/file.txt"):
        assert selected(name) is None, name
    archive = io.BytesIO()
    body = b'{"unchanged":true}\n'
    with tarfile.open(fileobj=archive, mode="w:gz") as writer:
        for _ in range(2):
            info = tarfile.TarInfo("cold-scale/base-1/measurement.json")
            info.size = len(body)
            writer.addfile(info, io.BytesIO(body))
        link = tarfile.TarInfo("cold-scale/builds/link.json")
        link.type = tarfile.SYMTYPE
        link.linkname = "/private"
        writer.addfile(link)
    archive.seek(0)
    output = {}
    with tarfile.open(fileobj=archive, mode="r|gz") as stream:
        manifest = collect(stream, output.__setitem__, required={"base.sha"})
    assert output == {"base-1/measurement.json": body}
    assert len(manifest["errors"]) == 2
    assert manifest["requiredMissing"] == ["base.sha"]
    assert manifest["totalBytes"] == len(body)
    print("thin metadata controls: exact bytes; binary/cache/data/traversal/link/duplicate rejected; missing evidence stays missing")


if __name__ == "__main__":
    self_test() if sys.argv[1:] == ["--self-test"] else main()
