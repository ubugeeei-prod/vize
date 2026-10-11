#!/usr/bin/env python3
"""Read-only authenticated artifact download without recording redirect credentials."""
import json
import os
import re
import subprocess
import time
from pathlib import Path

from artifact_payload_files import digest_file, json_bytes, require


def download(destination):
    started = time.monotonic()
    repository = os.environ["GITHUB_REPOSITORY"]
    artifact_id = os.environ["PAYLOAD_ARTIFACT_ID"]
    source_sha = os.environ["PAYLOAD_SOURCE_SHA"]
    require(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository), "invalid repository")
    require(re.fullmatch(r"[1-9][0-9]*", artifact_id), "invalid artifact ID")
    require(re.fullmatch(r"[0-9a-f]{40}", source_sha), "invalid source SHA")
    destination = Path(destination)
    require(not destination.exists(), "download destination must be fresh")
    endpoint = f"repos/{repository}/actions/artifacts/{artifact_id}"
    # gh follows cross-host redirects with its normal credential stripping. Never expose
    # stderr, debug output, headers, tokens, or the temporary signed redirect URL.
    environment = {**os.environ, "GH_DEBUG": ""}
    metadata_result = subprocess.run(["gh", "api", endpoint], capture_output=True, env=environment)
    require(metadata_result.returncode == 0, "artifact metadata request failed")
    metadata = json.loads(metadata_result.stdout)
    run = metadata["workflow_run"]
    require(not metadata["expired"] and metadata["id"] == int(artifact_id), "artifact unavailable")
    require(run["head_sha"] == source_sha, "original source identity differs")
    require(run["repository_id"] == run["head_repository_id"], "fork artifact is not admitted")
    require(re.fullmatch(r"sha256:[0-9a-f]{64}", metadata["digest"]), "artifact has no SHA-256 digest")
    destination.mkdir()
    (destination / "original-metadata.json").write_bytes(json_bytes(metadata))
    with (destination / "original.zip").open("xb") as archive:
        result = subprocess.run(["gh", "api", f"{endpoint}/zip"], stdout=archive,
                                stderr=subprocess.PIPE, env=environment)
    require(result.returncode == 0, "artifact body request failed")
    archive = destination / "original.zip"
    require(archive.stat().st_size == metadata["size_in_bytes"], "original archive size differs")
    require(f"sha256:{digest_file(archive)}" == metadata["digest"], "original archive digest differs")
    report = {
        "artifactId": int(artifact_id), "sourceSha": source_sha,
        "originalRunId": run["id"], "originalZipBytes": archive.stat().st_size,
        "downloadAndDigestMs": round((time.monotonic() - started) * 1000),
    }
    (destination / "download.json").write_bytes(json_bytes(report))
    print(json.dumps(report, sort_keys=True))


if __name__ == "__main__":
    import sys
    download(sys.argv[1])
