#!/usr/bin/env python3
"""Opt-in trial: preserve an entire authenticated artifact ZIP with SHA-256 dedup."""
import argparse
import hashlib
import json
import re
import shutil
import tempfile
import time
import zipfile
from pathlib import Path

from artifact_payload_files import (
    SCHEMA, digest_file, digest_stream, json_bytes, require, safe_name,
    validate_manifest, verify_tree, zip_row,
)

PAYLOAD = "objects.zip"
MANIFEST = "manifest.json"
RECEIPT = "receipt.json"


def milliseconds(start):
    return round((time.monotonic() - start) * 1000)


def pack(original, output, metadata, source_sha, driver_sha):
    started = time.monotonic()
    original, output = Path(original), Path(output)
    require(not output.exists(), "payload output must be fresh")
    for sha in (source_sha, driver_sha):
        require(re.fullmatch("[0-9a-f]{40}", sha), "invalid source or driver SHA")
    require(metadata["workflow_run"]["head_sha"] == source_sha, "original source identity differs")
    original_digest = digest_file(original)
    require(metadata["digest"] == f"sha256:{original_digest}", "original archive digest differs")
    require(metadata["size_in_bytes"] == original.stat().st_size, "original archive size differs")
    output.mkdir()
    provenance = {
        "schema": SCHEMA, "sourceSha": source_sha, "driverSha": driver_sha,
        "artifactId": metadata["id"], "originalZipSha256": original_digest,
    }
    entries, names, objects = [], set(), {}
    hashing_ms = compression_ms = 0
    with zipfile.ZipFile(original) as source, zipfile.ZipFile(
        output / PAYLOAD, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=6,
        allowZip64=True,
    ) as target, tempfile.TemporaryDirectory(prefix="vize-payload-body-") as temporary:
        body = Path(temporary) / "body"
        for info in source.infolist():
            row = zip_row(info)
            require(row["path"] not in names, "duplicate original path")
            names.add(row["path"])
            if row["kind"] == "file":
                hash_start = time.monotonic()
                with source.open(info) as stream, body.open("wb") as raw:
                    sha, size = digest_stream(stream, raw)
                require(size == info.file_size, "original body size differs")
                hashing_ms += (time.monotonic() - hash_start) * 1000
                row.update(sha256=sha, bytes=size)
                if sha not in objects:
                    compress_start = time.monotonic()
                    target.write(body, f"objects/{sha}")
                    compression_ms += (time.monotonic() - compress_start) * 1000
                    objects[sha] = size
                else:
                    require(objects[sha] == size, "same digest has different size")
            entries.append(row)
    manifest = {**provenance, "entries": sorted(entries, key=lambda row: row["path"])}
    manifest_bytes = json_bytes(manifest)
    (output / MANIFEST).write_bytes(manifest_bytes)
    receipt = {
        **provenance, "originalArtifactName": metadata["name"],
        "originalRunId": metadata["workflow_run"]["id"],
        "originalZipBytes": original.stat().st_size,
        "entryCount": len(entries), "fileCount": sum(row["kind"] == "file" for row in entries),
        "rawBytes": sum(row.get("bytes", 0) for row in entries),
        "objectCount": len(objects), "uniqueBodyBytes": sum(objects.values()),
        "payloadZipBytes": (output / PAYLOAD).stat().st_size,
        "manifestBytes": len(manifest_bytes),
        "payloadZipSha256": digest_file(output / PAYLOAD),
        "manifestSha256": hashlib.sha256(manifest_bytes).hexdigest(),
        "wholeBodyReadAndHashMs": round(hashing_ms), "uniqueObjectCompressionMs": round(compression_ms),
        "packMs": milliseconds(started), "compression": "deflate-6",
        "qualification": "transport trial only; original source acceptance is retained separately",
    }
    validate_manifest(manifest, receipt)
    (output / RECEIPT).write_bytes(json_bytes(receipt))
    return receipt


def restore(output, destination, source_sha, driver_sha, artifact_id):
    started = time.monotonic()
    output, destination = Path(output), Path(destination)
    require(not destination.exists(), "restore destination must be fresh")
    require({entry.name for entry in output.iterdir()} == {PAYLOAD, MANIFEST, RECEIPT},
            "payload envelope inventory differs")
    for name in (PAYLOAD, MANIFEST, RECEIPT):
        require((output / name).is_file() and not (output / name).is_symlink(), "invalid envelope file")
    receipt = json.loads((output / RECEIPT).read_bytes())
    require(receipt["sourceSha"] == source_sha and receipt["driverSha"] == driver_sha
            and receipt["artifactId"] == artifact_id, "transport source or driver identity differs")
    require(digest_file(output / PAYLOAD) == receipt["payloadZipSha256"], "damaged payload archive")
    require(digest_file(output / MANIFEST) == receipt["manifestSha256"], "damaged payload manifest")
    manifest = json.loads((output / MANIFEST).read_bytes())
    objects = validate_manifest(manifest, receipt)
    require(manifest["originalZipSha256"] == receipt["originalZipSha256"], "original digest differs")
    with zipfile.ZipFile(output / PAYLOAD) as source, tempfile.TemporaryDirectory(
        prefix="vize-payload-restore-", dir=destination.parent,
    ) as temporary:
        infos = source.infolist()
        require(len(infos) == len(objects)
                and {info.filename for info in infos} == {f"objects/{sha}" for sha in objects},
                "missing or unexpected payload object")
        for info in infos:
            sha = info.filename.removeprefix("objects/")
            with source.open(info) as stream, (Path(temporary) / sha).open("xb") as raw:
                actual_sha, size = digest_stream(stream, raw)
            require(actual_sha == sha and size == objects[sha], "damaged whole object body")
        destination.mkdir()
        for row in manifest["entries"]:
            path = destination / safe_name(row["path"])
            path.parent.mkdir(parents=True, exist_ok=True)
            if row["kind"] == "directory":
                path.mkdir(exist_ok=True)
            else:
                # Copies deliberately restore independent mutable files, including identical bodies.
                with (Path(temporary) / row["sha256"]).open("rb") as raw, path.open("xb") as target:
                    shutil.copyfileobj(raw, target, 1024 * 1024)
                path.chmod(row["mode"])
        # Apply directory modes after writing their children.
        for row in reversed(manifest["entries"]):
            if row["kind"] == "directory":
                (destination / row["path"]).chmod(row["mode"])
    verification = verify_tree(destination, manifest)
    return {**verification, "restoreAndFullVerifyMs": milliseconds(started),
            "sourceSha": source_sha, "driverSha": driver_sha, "artifactId": artifact_id}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("pack", "restore", "verify"))
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--metadata", type=Path)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--driver-sha", required=True)
    parser.add_argument("--artifact-id", type=int)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    if args.operation == "pack":
        result = pack(args.input, args.output, json.loads(args.metadata.read_bytes()),
                      args.source_sha, args.driver_sha)
    elif args.operation == "restore":
        result = restore(args.input, args.output, args.source_sha, args.driver_sha, args.artifact_id)
    else:
        result = verify_tree(args.input, json.loads(args.output.read_bytes()))
    if args.report:
        args.report.write_bytes(json_bytes(result))
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
