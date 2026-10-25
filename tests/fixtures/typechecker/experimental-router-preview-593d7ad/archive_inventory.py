"""Read a package archive without extracting or trusting archive paths."""

import hashlib
import json
import sys
import tarfile
from pathlib import PurePosixPath


def inventory(archive):
    result = []
    seen = set()
    with tarfile.open(archive, "r:gz") as source:
        for member in source:
            name = member.name[:-1] if member.isdir() and member.name.endswith("/") else member.name
            parts = name.split("/")
            if (
                name.startswith("/")
                or parts[0] != "package"
                or any(part in ("", ".", "..") for part in parts)
            ):
                raise ValueError(f"unsafe package member: {member.name!r}")
            relative = PurePosixPath(*parts[1:]).as_posix()
            if member.isdir():
                continue
            if not member.isfile() or len(parts) < 2:
                raise ValueError(f"non-regular package member: {member.name!r}")
            if relative in seen:
                raise ValueError(f"duplicate package member: {relative!r}")
            seen.add(relative)
            stream = source.extractfile(member)
            if stream is None:
                raise ValueError(f"missing package bytes: {relative!r}")
            data = stream.read()
            if len(data) != member.size:
                raise ValueError(f"incomplete package bytes: {relative!r}")
            result.append(
                {
                    "path": relative,
                    "bytes": len(data),
                    "sha256": hashlib.sha256(data).hexdigest(),
                }
            )
    if not result:
        raise ValueError("empty package archive")
    return sorted(result, key=lambda entry: entry["path"])


if __name__ == "__main__":
    try:
        json.dump(inventory(sys.argv[1]), sys.stdout, indent=2)
        sys.stdout.write("\n")
    except ValueError as error:
        json.dump({"error": str(error)}, sys.stderr)
        sys.stderr.write("\n")
        sys.exit(1)
