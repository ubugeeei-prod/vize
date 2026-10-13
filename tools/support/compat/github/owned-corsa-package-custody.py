#!/usr/bin/env python3
"""Authenticate and stage Cargo's six generated archives; never compile source."""
import hashlib
import copy
import json
import os
from pathlib import Path, PurePosixPath
import stat
import subprocess
import sys
import tarfile

NAMES = ("corsa_core", "corsa_jsonrpc", "corsa_client", "corsa_lsp", "corsa_orchestrator", "corsa")
MANIFEST_SHA = "bffe80f1e770ad10df3af2670e01b5f53a9574f357d42df6948a458f5311040c"
GENERATED = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", ".cargo_vcs_info.json"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(body):
    return hashlib.sha256(body).hexdigest()


def regular(path):
    metadata = path.lstat()
    require(stat.S_ISREG(metadata.st_mode) and stat.S_IMODE(metadata.st_mode) == 0o644, str(path))
    return path.read_bytes()


def toml(root, body):
    parser = "import fs from 'node:fs';import {createRequire} from 'node:module';const r=createRequire(process.argv[1]);process.stdout.write(JSON.stringify(r('@iarna/toml').parse(fs.readFileSync(0,'utf8'))));"
    result = subprocess.run([os.environ.get("NODE_EXE", "node"), "--input-type=module", "-e", parser, str(root / "package.json")], input=body, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    require(result.returncode == 0, result.stderr.decode(errors="replace"))
    return json.loads(result.stdout)


def normalized_manifest(source, root_manifest, name):
    expected = copy.deepcopy(source)
    version = root_manifest["workspace"]["package"]["version"]
    require(name in NAMES and version == "0.443.0", "finite graph identity/version")
    require(expected["package"]["version"] == {"workspace": True}, "workspace version")
    expected["package"]["version"] = version
    require(expected["package"]["resolver"] == "2" and root_manifest["workspace"]["resolver"] == "3", "finite source/workspace resolver pair")
    expected["package"]["resolver"] = "3"
    require(expected["package"]["name"] == "vize_" + name, "owned package identity")
    require(expected["package"]["exclude"] == [".cargo_vcs_info.json", "Cargo.toml.orig"], "reserved source files retained but regenerated")
    require(expected["lib"]["name"] == name, "Rust library alias")
    for key, dependency in expected.get("dependencies", {}).items():
        if key in NAMES:
            require(dependency == {"workspace": True}, "internal workspace edge")
            inherited = root_manifest["workspace"]["dependencies"][key].copy()
            require(inherited == {"package": "vize_" + key, "version": "=" + version, "path": "vendor/" + key}, "root alias/version/path cell")
            inherited.pop("path")
            expected["dependencies"][key] = inherited
    return expected


def main(root, package_directory, vendor, manifest_path, source_sha, output):
    require(len(source_sha) == 40 and all(c in "0123456789abcdef" for c in source_sha), "source SHA")
    identity = subprocess.check_output(["git", "--no-replace-objects", "rev-parse", "HEAD"], cwd=root).decode().strip()
    require(identity == source_sha, "actual raw source identity")
    raw = regular(manifest_path)
    require(sha(raw) == MANIFEST_SHA, "finite source manifest authority")
    data = json.loads(raw)
    parts = data["parts"]
    require([part["file"] for part in parts] == ["owned-corsa-source-part-" + str(index).zfill(2) + ".json" for index in range(20)], "finite source parts")
    rows = []
    for part in parts:
        body = regular(manifest_path.parent / part["file"])
        require(sha(body) == part["sha256"], "complete authority part: " + part["file"])
        values = json.loads(body)
        require(len(values) == part["bodyPairs"], "authority part count")
        rows.extend(values)
    require(len(rows) == 237 and len({row["file"] for row in rows}) == 237, "complete source census")
    verifier = root / "tools/support/compat/github/owned-corsa-package-source.mjs"
    require(sha(regular(verifier)) == "a9ad6942e5e82583ae6f741ef63f4f8ebcf4100a2265c3a8f04502e9a1b8eb30", "whole shared source verifier")
    result = subprocess.run([os.environ.get("NODE_EXE", "node"), str(verifier), str(root), source_sha], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, env={**os.environ, "GIT_NO_LAZY_FETCH": "1"})
    require(result.returncode == 0, result.stderr.decode(errors="replace"))
    source_authority = json.loads(result.stdout)
    require(source_authority["head"] == source_sha and source_authority["manifestSHA256"] == MANIFEST_SHA and source_authority["completeProductionBodies"] == 237, "same complete actual source authority")
    cells = source_authority["actualBodies"]
    require([cell["file"] for cell in cells] == [row["file"] for row in rows], "exact source tuple order/census")
    for cell in cells:
        body = regular(root / cell["file"])
        blob = hashlib.sha1(("blob " + str(len(body)) + "\0").encode() + body).hexdigest()
        entry = subprocess.check_output(["git", "--no-replace-objects", "ls-tree", "-z", source_sha, "--", cell["file"]], cwd=root)
        require(cell["mode"] == "100644" and cell["bytes"] == len(body) and cell["sha256"] == sha(body) and cell["blob"] == blob, cell["file"])
        require(entry == ("100644 blob " + blob + "\t" + cell["file"] + "\0").encode(), "actual source mode/blob: " + cell["file"])
    root_manifest = toml(root, regular(root / "Cargo.toml"))
    version = root_manifest["workspace"]["package"]["version"]
    require(version == "0.443.0", "this finite package graph version")
    original_runtime = "8f812d48ff665d41cae4411438dfa30fff35429eb78ad1aa698317fd477e77da"
    staged = []
    for name in NAMES:
        source = root / "vendor" / name
        expected_files = {row["file"].split("/", 2)[2] for row in rows if row["file"].startswith("vendor/" + name + "/")}
        actual_files = set()
        for path in source.rglob("*"):
            require(not path.is_symlink(), "symlink source: " + str(path))
            if path.is_file():
                actual_files.add(path.relative_to(source).as_posix())
        require(actual_files == expected_files, "unknown/missing source member: " + name)
        owned = "vize_" + name
        archive = package_directory / (owned + "-" + version + ".crate")
        archive_bytes = regular(archive)
        bodies = {}
        with tarfile.open(archive, "r:gz") as tar:
            for member in tar.getmembers():
                prefix = owned + "-" + version + "/"
                require(member.isfile() and member.mode == 0o644 and member.name.startswith(prefix), "nonregular or foreign archive member")
                relative = member.name[len(prefix):]
                pieces = PurePosixPath(relative).parts
                require(relative and not relative.startswith("/") and ".." not in pieces and "." not in pieces and str(PurePosixPath(relative)) == relative, "archive path")
                require(relative not in bodies, "duplicate archive member")
                bodies[relative] = tar.extractfile(member).read()
        require(set(bodies) == expected_files, "unknown/missing packaged member: " + name)
        for relative, body in bodies.items():
            if relative not in GENERATED:
                require(body == regular(source / relative), "whole package source body: " + relative)
        require(bodies["Cargo.toml.orig"] == regular(source / "Cargo.toml"), "current original manifest")
        vcs = json.loads(bodies[".cargo_vcs_info.json"])
        require(vcs in [{"git": {"sha1": source_sha}, "path_in_vcs": "vendor/" + name}, {"git": {"sha1": source_sha, "dirty": False}, "path_in_vcs": "vendor/" + name}], "generated clean VCS identity is actual source, never original archive provenance")
        expected = normalized_manifest(toml(root, regular(source / "Cargo.toml")), root_manifest, name)
        require(toml(root, bodies["Cargo.toml"]) == expected, "entire normalized manifest, not independent fields")
        lock = toml(root, bodies["Cargo.lock"])
        packages = lock["package"]
        require(not any(package["name"] in NAMES for package in packages), "registry upstream core graph must be absent")
        runtimes = [package for package in packages if package["name"] == "corsa_runtime"]
        if name != "corsa_core":
            require(len(runtimes) == 1 and runtimes[0]["version"] == "1.14.0" and runtimes[0]["checksum"] == original_runtime, "published runtime exact")
        destination = vendor / (owned + "-" + version)
        require(not destination.exists() and not destination.is_symlink(), "do not overwrite sealed directory")
        destination.mkdir()
        members = []
        for relative, body in sorted(bodies.items()):
            path = destination / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(body)
            path.chmod(0o644)
            members.append({"file": relative, "mode": "100644", "bytes": len(body), "sha256": sha(body), "generated": relative in GENERATED})
        checksum = {"package": sha(archive_bytes), "files": {item["file"]: item["sha256"] for item in members}}
        (destination / ".cargo-checksum.json").write_text(json.dumps(checksum, sort_keys=True) + "\n")
        staged.append({"name": owned, "version": version, "archive": str(archive), "archiveBytes": len(archive_bytes), "archiveSHA256": sha(archive_bytes), "members": members})
    output.write_text(json.dumps({"sourceSha": source_sha, "finiteSourceManifestSHA256": sha(raw), "completeSourceBodies": 237, "sourceContext": source_authority["sourceContext"], "packages": staged, "buildVerified": False, "publicRegistryAcceptance": False}, indent=2) + "\n")


if __name__ == "__main__":
    require(len(sys.argv) == 7, "root package_directory vendor manifest source_sha output")
    main(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]), sys.argv[5], Path(sys.argv[6]))
