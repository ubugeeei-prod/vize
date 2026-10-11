"""Whole-body identity and independent-file restoration for the opt-in ZIP trial."""
import hashlib
import json
import os
import stat
from pathlib import Path, PurePosixPath

CHUNK = 1024 * 1024
SCHEMA = "vize.artifact-payload.v1"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest_stream(stream, output=None):
    digest = hashlib.sha256()
    size = 0
    while chunk := stream.read(CHUNK):
        digest.update(chunk)
        size += len(chunk)
        if output is not None:
            output.write(chunk)
    return digest.hexdigest(), size


def digest_file(path):
    with open(path, "rb") as stream:
        return digest_stream(stream)[0]


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n").encode()


def safe_name(name, directory=False):
    require(isinstance(name, str) and name and "\x00" not in name, "invalid path")
    raw = name[:-1] if directory and name.endswith("/") else name
    parts = raw.split("/")
    require(
        not PurePosixPath(raw).is_absolute()
        and not any(part in ("", ".", "..") for part in parts)
        and "\\" not in raw,
        "unsafe artifact path",
    )
    return raw


def zip_row(info):
    directory = info.is_dir()
    name = safe_name(info.filename, directory)
    mode = info.external_attr >> 16
    require(
        stat.S_IFMT(mode) in (0, stat.S_IFDIR if directory else stat.S_IFREG),
        "unsupported original ZIP entry type",
    )
    require(not directory or info.file_size == 0, "directory contains file bytes")
    return {
        "path": name,
        "kind": "directory" if directory else "file",
        "mode": stat.S_IMODE(mode) if mode else (0o755 if directory else 0o644),
        # Retain the entire original permission/type metadata, even where ZIP has none.
        "externalAttr": info.external_attr,
        "createSystem": info.create_system,
    }


def validate_manifest(manifest, receipt):
    for key in ("schema", "sourceSha", "driverSha", "artifactId"):
        require(manifest[key] == receipt[key], "manifest provenance differs")
    require(manifest["schema"] == SCHEMA, "unsupported payload schema")
    entries = manifest["entries"]
    require(len(entries) == receipt["entryCount"], "manifest entry count differs")
    names = set()
    objects = {}
    for row in entries:
        require(row["kind"] in ("file", "directory"), "invalid entry kind")
        name = safe_name(row["path"])
        require(name not in names, "duplicate artifact path")
        names.add(name)
        require(type(row["mode"]) is int and 0 <= row["mode"] <= 0o7777, "invalid mode")
        if row["kind"] == "file":
            sha = row["sha256"]
            require(len(sha) == 64 and all(c in "0123456789abcdef" for c in sha), "invalid digest")
            require(type(row["bytes"]) is int and row["bytes"] >= 0, "invalid byte count")
            require(sha not in objects or objects[sha] == row["bytes"], "object size differs")
            objects[sha] = row["bytes"]
    file_names = {row["path"] for row in entries if row["kind"] == "file"}
    for name in names:
        require(not any(str(parent) in file_names for parent in PurePosixPath(name).parents),
                "file shadows parent directory")
    require(sum(row.get("bytes", 0) for row in entries) == receipt["rawBytes"], "raw byte count differs")
    require(len(objects) == receipt["objectCount"], "object count differs")
    return objects


def verify_tree(root, manifest):
    root = Path(root)
    require(stat.S_ISDIR(root.lstat().st_mode), "restored root must be an independent directory")
    expected = {row["path"]: row for row in manifest["entries"]}
    implicit = set()
    for name in expected:
        implicit.update(str(parent) for parent in PurePosixPath(name).parents if str(parent) != ".")
    actual = set()
    inodes = set()
    def refuse_walk_error(error):
        raise ValueError("cannot inspect complete restored inventory") from error

    for directory, directories, files in os.walk(root, followlinks=False, onerror=refuse_walk_error):
        for name in directories + files:
            path = Path(directory) / name
            logical = path.relative_to(root).as_posix()
            metadata = path.lstat()
            require(not stat.S_ISLNK(metadata.st_mode), "restored symlink is forbidden")
            require(logical in expected or logical in implicit, "unexpected restored path")
            if logical not in expected:
                require(stat.S_ISDIR(metadata.st_mode), "implicit parent is not a directory")
                continue
            actual.add(logical)
            row = expected[logical]
            require(stat.S_IMODE(metadata.st_mode) == row["mode"], "restored mode differs")
            if row["kind"] == "directory":
                require(stat.S_ISDIR(metadata.st_mode), "restored directory differs")
            else:
                require(stat.S_ISREG(metadata.st_mode), "restored file differs")
                require(metadata.st_nlink == 1, "restored file has mutable aliases")
                identity = (metadata.st_dev, metadata.st_ino)
                require(identity not in inodes, "restored files share an inode")
                inodes.add(identity)
                require(metadata.st_size == row["bytes"] and digest_file(path) == row["sha256"],
                        "restored file body differs")
    require(actual == set(expected), "missing restored path")
    return {"entries": len(actual), "independentFiles": len(inodes)}
