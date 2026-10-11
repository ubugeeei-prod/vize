#!/usr/bin/env python3
"""Frozen-source record-only compression trial; never rewrite the build receipt."""
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import time
import zipfile
from pathlib import Path

from artifact_payload_files import digest_file, json_bytes, require
from rust_archive_inventory import inventory

ARTIFACT = 11706049605
API_SOURCE = "1f463e566cee76d093b3f0c41ee30b7bd1314cdb"
SOURCE = "e057b3dd9e8ebed0dee7f4239a34daa2b410f7a0"
TREE = "133c282d0c7404fddaf8ce7d3d159ac6cd3d5a86"
ZIP_SHA = "6eefdf642a5d28e83219f74db7886d474f29842202765b88c9e1023832f75707"
ARCHIVE_SHA = "496256643c438487267a6e4ddb6cb9963a57d98c6e30a6678e72173e0e0eef37"
ZIP_BYTES = 4821748890
WORKSPACE = "/home/runner/_work/vize/vize"
ENCODER = ["zstd", "-3", "--long=27", "-T4", "-c"]
DRIVER_FILES = ("artifact_payload_files.py", "artifact_payload_download.py",
                "rust_archive_inventory.py", "rust_archive_transport.py",
                "rust_archive_workers.py", "rust_archive_controls.py")


def driver_files(directory):
    directory = Path(directory)
    require({path.name for path in directory.glob("rust_archive_*.py")} ==
            {name for name in DRIVER_FILES if name.startswith("rust_archive_")},
            "driver helper inventory differs")
    result = {}
    for name in DRIVER_FILES:
        path = directory / name
        info = path.lstat()
        require(stat.S_ISREG(info.st_mode), "driver helper is not an independent regular file")
        result[name] = {"sha256": digest_file(path), "mode": stat.S_IMODE(info.st_mode)}
    return result


def authenticate_driver(driver):
    def git(*args):
        return subprocess.check_output(["git", "--no-replace-objects", *args])
    require(re.fullmatch(r"[0-9a-f]{40}", driver)
            and git("rev-parse", "HEAD").decode().strip() == driver, "driver checkout differs")
    result = driver_files(Path(__file__).parent)
    for name, entry in result.items():
        path = f"tools/support/compat/github/{name}"
        require(hashlib.sha256(git("show", f"{driver}:{path}")).hexdigest() == entry["sha256"]
                and git("ls-tree", driver, path).decode().startswith("100644 ")
                and entry["mode"] == 0o644, "driver helper body/mode differs from its real Git source")
    return {"driverSha": driver, "files": result}


def validate_receipt(receipt):
    expected = {"schemaVersion": 3, "workspaceRoot": WORKSPACE, "sha": SOURCE,
                "tree": TREE, "platform": "linux", "arch": "x64",
                "nextestVersion": "0.9.146", "cargoProfile": "ci",
                "requireTsgo": None, "disableTsgo": "1", "nuxtIterations": "100",
                "archiveSha256": ARCHIVE_SHA}
    require(set(receipt) == set(expected) | {"rustcVersion"}, "original receipt keys differ")
    require(all(receipt[key] == value for key, value in expected.items()),
            "original receipt identity differs")
    require(receipt["rustcVersion"] == (
        "rustc 1.99.0 (b940084d7 2026-09-28)\nbinary: rustc\ncommit-hash: "
        "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4\ncommit-date: 2026-09-28\n"
        "host: x86_64-unknown-linux-gnu\nrelease: 1.99.0\nLLVM version: 23.1.1"),
        "original compiler identity differs")


def unpack_original(original, destination):
    original, destination = Path(original), Path(destination)
    require(original.stat().st_size == ZIP_BYTES and digest_file(original) == ZIP_SHA,
            "complete original ZIP identity differs")
    require(not destination.exists(), "original extraction must be fresh")
    destination.mkdir(parents=True)
    with zipfile.ZipFile(original) as archive:
        entries = archive.infolist()
        require(len(entries) == 2 and {entry.filename for entry in entries} ==
                {"receipt.json", "tests.tar.zst"}, "original ZIP inventory differs")
        for entry in entries:
            require(entry.compress_type == zipfile.ZIP_STORED, "original ZIP encoding differs")
            # Reading to EOF verifies each complete ZIP member CRC as well as the ZIP SHA.
            with archive.open(entry) as reader, (destination / entry.filename).open("xb") as writer:
                shutil.copyfileobj(reader, writer, 1024 * 1024)
    receipt = json.loads((destination / "receipt.json").read_bytes())
    validate_receipt(receipt)
    require(digest_file(destination / "tests.tar.zst") == ARCHIVE_SHA,
            "original built archive digest differs")
    return receipt


def transcode(original, candidate, expected_tar_sha, zstd="zstd"):
    started = time.monotonic()
    require(not Path(candidate).exists(), "candidate must be fresh")
    digest = hashlib.sha256()
    command = [zstd, *ENCODER[1:]]
    with Path(candidate).open("xb") as output, Path(candidate).with_suffix(".stderr").open("xb") as errors:
        decoder = subprocess.Popen([zstd, "-d", "-c", str(original)], stdout=subprocess.PIPE,
                                   stderr=errors)
        encoder = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=output, stderr=errors)
        try:
            while chunk := decoder.stdout.read(1024 * 1024):
                digest.update(chunk)
                encoder.stdin.write(chunk)
            encoder.stdin.close()
            require(decoder.wait() == 0 and encoder.wait() == 0, "complete transcode failed")
        finally:
            decoder.stdout.close()
            for process in (decoder, encoder):
                if process.poll() is None:
                    process.kill()
                    process.wait()
    require(digest.hexdigest() == expected_tar_sha, "transcode decoded tar identity differs")
    return {"command": command, "elapsedMs": round((time.monotonic() - started) * 1000)}


def prepare(download, destination, driver):
    download, destination = Path(download), Path(destination)
    require(re.fullmatch(r"[0-9a-f]{40}", driver), "invalid driver SHA")
    require(not destination.exists(), "transport destination must be fresh")
    driver_receipt = authenticate_driver(driver)
    metadata = json.loads((download / "original-metadata.json").read_bytes())
    require(metadata["id"] == ARTIFACT and metadata["workflow_run"]["head_sha"] == API_SOURCE
            and metadata["digest"] == f"sha256:{ZIP_SHA}" and metadata["size_in_bytes"] == ZIP_BYTES,
            "authenticated API original identity differs")
    destination.mkdir()
    baseline = destination / "baseline"
    unpack_original(download / "original.zip", baseline)
    def measured_inventory(archive, name):
        path = destination / f"{name}-inventory.json"
        command = ["/usr/bin/time", "-v", "-o", str(destination / f"{name}-decode.resources"),
                   sys.executable, "-B", str(Path(__file__).with_name("rust_archive_inventory.py")),
                   str(archive), str(path)]
        with (destination / f"{name}-decode.stdout").open("xb") as output:
            require(subprocess.run(command, stdout=output).returncode == 0, "inventory process failed")
        return json.loads(path.read_bytes()), json.loads(path.with_suffix(".summary.json").read_bytes())
    before, before_summary = measured_inventory(baseline / "tests.tar.zst", "baseline")
    candidate = destination / "candidate"
    candidate.mkdir()
    encode = transcode(baseline / "tests.tar.zst", candidate / "tests.tar.zst", before["tarSha256"])
    after, after_summary = measured_inventory(candidate / "tests.tar.zst", "candidate")
    require(before == after, "complete decoded tar/member/body/type/mode identity differs")
    shutil.copyfile(baseline / "receipt.json", destination / "original-receipt.json")
    report = {"schemaVersion": 1, "driverSha": driver, "sourceSha": SOURCE,
              "sourceTree": TREE, "apiSourceSha": API_SOURCE, "artifactId": ARTIFACT,
              "driverFiles": driver_receipt["files"],
              "originalZipSha256": ZIP_SHA, "originalZipBytes": ZIP_BYTES,
              "baseline": before_summary, "candidate": after_summary, "encoder": encode,
              "zstdVersion": subprocess.check_output(["zstd", "--version"], text=True).strip(),
              "zstdExecutableSha256": digest_file(shutil.which("zstd")),
              "qualification": "record-only PR-runtime transport trial; original full gates retained separately"}
    (destination / "transport.json").write_bytes(json_bytes(report))
    print(json.dumps(report, sort_keys=True))


def verify(custody, arm, archive, driver):
    custody = Path(custody)
    require(arm in ("baseline", "candidate"), "unknown transport arm")
    report = json.loads((custody / "transport.json").read_bytes())
    require(report["schemaVersion"] == 1 and report["driverSha"] == driver
            and report["sourceSha"] == SOURCE and report["sourceTree"] == TREE
            and report["apiSourceSha"] == API_SOURCE and report["artifactId"] == ARTIFACT
            and report["originalZipSha256"] == ZIP_SHA and report["originalZipBytes"] == ZIP_BYTES,
            "transport provenance differs")
    validate_receipt(json.loads((custody / "original-receipt.json").read_bytes()))
    require(report["driverFiles"] == driver_files(Path(__file__).parent),
            "staged driver helper body/mode differs")
    require(report["encoder"]["command"] == ENCODER, "bounded encoder arguments differ")
    before_path, after_path = custody / "baseline-inventory.json", custody / "candidate-inventory.json"
    require(digest_file(before_path) == report["baseline"]["inventorySha256"]
            and digest_file(after_path) == report["candidate"]["inventorySha256"],
            "transport inventory digest differs")
    before, after = json.loads(before_path.read_bytes()), json.loads(after_path.read_bytes())
    require(before == after and report["baseline"]["archiveSha256"] == ARCHIVE_SHA,
            "original/candidate inventory identity differs")
    require(before["tarSha256"] == report["baseline"]["tarSha256"] == report["candidate"]["tarSha256"]
            and before["tarBytes"] == report["baseline"]["tarBytes"] == report["candidate"]["tarBytes"]
            and len(before["members"]) == report["baseline"]["memberCount"] == report["candidate"]["memberCount"],
            "transport summary identity differs")
    require(digest_file(archive) == report[arm]["archiveSha256"]
            and Path(archive).stat().st_size == report[arm]["archiveBytes"], "transport body differs")
    return report


if __name__ == "__main__":
    import sys
    if sys.argv[1] == "prepare":
        prepare(*sys.argv[2:])
    elif sys.argv[1] == "unpack":
        unpack_original(*sys.argv[2:])
    elif sys.argv[1] == "verify":
        print(json.dumps(verify(*sys.argv[2:]), sort_keys=True))
    elif sys.argv[1] == "driver":
        Path(sys.argv[3]).write_bytes(json_bytes(authenticate_driver(sys.argv[2])))
    else:
        raise ValueError("unknown transport command")
