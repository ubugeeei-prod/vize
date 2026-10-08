"""Read public archives without extraction; compare the entire installed file set."""

import hashlib
import io
import os
import pathlib
import tarfile

from identity import digest, exact_path
from registry import sri_sha512


def installed_files(package_dir):
    files = []
    for parent, directories, names in os.walk(package_dir, followlinks=False):
        for name in directories + names:
            entry = pathlib.Path(parent) / name
            if entry.is_symlink() or (not entry.is_dir() and not entry.is_file()):
                raise ValueError("installed package contains a redirect/special file: " + str(entry))
        files.extend(pathlib.Path(parent) / name for name in names)
    return files


def archive_comparison(package_dir, payload, integrity):
    exact_path(str(package_dir), directory=True)
    if hashlib.sha512(payload).digest() != sri_sha512(integrity):
        raise ValueError("public tarball integrity differs")
    files = []
    names = set()
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:gz") as archive:
        for member in archive.getmembers():
            raw = member.name.removesuffix("/") if member.isdir() else member.name
            parts = raw.split("/")
            if (not parts or parts[0] != "package" or any(part in ("", ".", "..") for part in parts)
                    or "\\" in raw or (not member.isdir() and not member.isfile())):
                raise ValueError("unexpected registry tarball member: " + member.name)
            if member.isdir():
                continue
            if len(parts) < 2:
                raise ValueError("archive file must have a package-relative name")
            relative = "/".join(parts[1:])
            if relative in names:
                raise ValueError("duplicate public archive file: " + relative)
            names.add(relative)
            installed = exact_path(str(package_dir / relative), owner=package_dir)
            data = installed.read_bytes()
            if data != archive.extractfile(member).read():
                raise ValueError("installed bytes differ from actual public archive: " + relative)
            files.append({"path": str(installed), "sha256": digest(data), "bytes": len(data)})
    actual = {str(file.relative_to(package_dir)) for file in installed_files(package_dir)}
    if not names or actual != names:
        raise ValueError("public archive and installed package file sets differ")
    return {"fileCount": len(names), "tarballSHA512": hashlib.sha512(payload).hexdigest(),
            "allInstalledFileBytesEqualPublicArchive": True,
            "files": sorted(files, key=lambda item: item["path"])}


def recheck_files(receipts):
    for receipt in receipts:
        package_dir = pathlib.Path(receipt["packageDirectory"])
        files = installed_files(package_dir)
        expected = {item["path"] for item in receipt["files"]}
        if {str(file) for file in files} != expected:
            raise ValueError("public installed file set changed during collection")
        for item in receipt["files"]:
            if digest(exact_path(item["path"], owner=package_dir).read_bytes()) != item["sha256"]:
                raise ValueError("public installed bytes changed during collection")
