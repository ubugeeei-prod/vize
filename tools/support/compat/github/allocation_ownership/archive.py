"""Authenticate original hosted archives; retain only selected ELF and metadata."""

import collections
import json
import re
import shutil
import subprocess
import tarfile
import zipfile
from pathlib import Path
from common import BINARY, FEATURE, REPO, api, require, sha256, write_json


def download(source, kind, directory):
    artifact_id = source[f"{kind}_id"]
    metadata = api(f"actions/artifacts/{artifact_id}")
    expected_name = (f"rust-test-archive-{source['run']}-{source['sha']}" if kind == "archive"
                     else f"rust-test-timings-{source['run']}-1-{source['sha']}")
    require(metadata["id"] == artifact_id and metadata["name"] == expected_name and
            not metadata["expired"] and metadata["workflow_run"]["id"] == source["run"] and
            metadata["digest"] == f"sha256:{source[f'{kind}_zip_sha256']}",
            "official artifact identity/digest must match frozen input")
    path = directory / f"{kind}.zip"
    with path.open("wb") as stream:
        subprocess.run(["gh", "api", f"repos/{REPO}/actions/artifacts/{artifact_id}/zip"],
                       stdout=stream, check=True)
    require(sha256(path) == source[f"{kind}_zip_sha256"], "official complete ZIP digest")
    write_json(directory / f"{kind}-official.json", metadata)
    return path


def extract_zip(path, directory):
    with zipfile.ZipFile(path) as archive:
        entries = []
        for member in archive.infolist():
            name = Path(member.filename)
            require(not name.is_absolute() and ".." not in name.parts, "unsafe artifact path")
            if member.is_dir():
                continue
            destination = directory / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            with archive.open(member) as src, destination.open("wb") as dst:
                shutil.copyfileobj(src, dst)
            entries.append({"name": member.filename, "bytes": member.file_size,
                            "sha256": sha256(destination)})
    write_json(directory / "zip-members.json", entries)
    path.unlink()


def feature_units(html):
    match = re.search(r"const UNIT_DATA = (\[.*?\]);", Path(html).read_text(), re.DOTALL)
    require(match is not None, "actual Cargo UNIT_DATA feature inputs required")
    units = json.loads(match[1])
    require(len(units) > 900 and any(unit["name"] == "davinci_harness" for unit in units),
            "complete workspace Cargo compiler-unit envelope required")
    return [{key: unit[key] for key in ("name", "version", "target", "features")}
            for unit in units]


def compare_units(original, instrumented):
    def normalized(units, overlay):
        result = collections.Counter()
        for unit in units:
            features = sorted(unit["features"])
            if overlay and unit["name"] == "davinci_harness":
                require(FEATURE in features, "explicit harness-only diagnostic compiler feature")
                features.remove(FEATURE)
            elif overlay:
                require(FEATURE not in features, "diagnostic feature cannot leak to production")
            result[(unit["name"], unit["version"], unit["target"], tuple(features))] += 1
        return result
    require(normalized(original, False) == normalized(instrumented, True),
            "every compiler-unit feature input must match except the explicit harness feature")


def select_tar(archive, destination):
    destination.mkdir(parents=True, exist_ok=True)
    retained = []
    process = subprocess.Popen(["zstd", "-dc", str(archive)], stdout=subprocess.PIPE)
    try:
        with tarfile.open(fileobj=process.stdout, mode="r|") as stream:
            for member in stream:
                name = Path(member.name)
                require(not name.is_absolute() and ".." not in name.parts, "unsafe archive member")
                base = name.name
                selected = (base in ("cargo-metadata.json", "binaries-metadata.json") or
                            re.fullmatch(rf"{BINARY}-[0-9a-f]+", base) or
                            re.fullmatch(r"davinci_harness-[0-9a-f]+", base))
                if not selected:
                    continue
                require(member.isfile(), "selected archive member must be a regular file")
                target = destination / name
                target.parent.mkdir(parents=True, exist_ok=True)
                with stream.extractfile(member) as src, target.open("wb") as dst:
                    shutil.copyfileobj(src, dst)
                target.chmod(member.mode)
                retained.append({"member": member.name, "bytes": member.size,
                                 "sha256": sha256(target)})
        require(process.wait() == 0, "complete zstd archive stream")
    finally:
        if process.poll() is None:
            process.kill()
    write_json(destination / "retained-members.json", retained)
    budgets = [destination / row["member"] for row in retained
               if re.fullmatch(rf"{BINARY}-[0-9a-f]+", Path(row["member"]).name)]
    harnesses = [destination / row["member"] for row in retained
                if re.fullmatch(r"davinci_harness-[0-9a-f]+", Path(row["member"]).name)]
    require(len(budgets) == 1 and len(harnesses) == 1, "one actual budget and harness ELF required")
    require(budgets[0].read_bytes()[:4] == b"\x7fELF", "actual Linux ELF")
    metadata = [destination / row["member"] for row in retained
                if Path(row["member"]).name == "cargo-metadata.json"]
    binary_meta = [destination / row["member"] for row in retained
                   if Path(row["member"]).name == "binaries-metadata.json"]
    require(len(metadata) == len(binary_meta) == 1, "actual archived build metadata paths required")
    # Cargo metadata retains the full resolve graph and its effective features.
    cargo = json.loads(metadata[0].read_text())
    require(cargo.get("resolve", {}).get("nodes"), "full archived feature resolve graph")
    write_json(destination / "effective-resolve-features.json",
               [{"id": node["id"], "features": node["features"]}
                for node in cargo["resolve"]["nodes"]])
    return budgets[0], harnesses[0]


def original_archive(source, destination):
    destination.mkdir(parents=True, exist_ok=True)
    run = api(f"actions/runs/{source['run']}")
    require(run["head_sha"] == source["sha"] and run["event"] == "merge_group" and
            run["status"] == "completed", "literal protected run source identity")
    write_json(destination / "official-run.json", run)
    timing = destination / "timing"
    timing.mkdir()
    extract_zip(download(source, "timing", timing), timing)
    original_units = feature_units(timing / "cargo-timing.html")
    write_json(destination / "compiler-feature-inputs.json", original_units)
    packet = destination / "archive"
    packet.mkdir()
    extract_zip(download(source, "archive", packet), packet)
    receipt = json.loads((packet / "receipt.json").read_text())
    require(receipt == json.loads((timing / "archive-receipt.json").read_text()),
            "owning timing and actual archive receipts must agree")
    require(receipt["schemaVersion"] == 3 and receipt["sha"] == source["sha"] and
            receipt["cargoProfile"] == "ci" and receipt["nextestVersion"] == "0.9.146" and
            receipt["rustcVersion"].startswith("rustc 1.98.0 ") and
            receipt["platform"] == "linux" and receipt["arch"] == "x64" and
            receipt["requireTsgo"] == "1" and receipt["disableTsgo"] is None and
            receipt["nuxtIterations"] == "100", "original full compiler/runtime envelope")
    archive = packet / "tests.tar.zst"
    require(sha256(archive) == receipt["archiveSha256"], "actual stamped full archive digest")
    budget, harness = select_tar(archive, destination / "selected")
    archive.unlink()
    suite = json.loads((timing / "workspace-tests.json").read_text())
    law = suite["rust-suites"][f"vize_atelier_vapor::{BINARY}"]
    require(Path(law["binary-path"]).name == budget.name and suite["test-count"] == 15519,
            "original 15,519-case inventory must select that actual budget ELF")
    return budget, harness, original_units
