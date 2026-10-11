"""Inspect one public CLI member before writing a self-contained GNU consumer."""

import hashlib
import json
import pathlib
import sys
import tarfile


def extract(archive, destination):
    destination = pathlib.Path(destination)
    if destination.exists():
        raise ValueError("public distribution destination must be fresh")
    with tarfile.open(archive, "r:gz") as stream:
        members = stream.getmembers()
        if len(members) != 1 or members[0].name != "vize" or not members[0].isreg():
            raise ValueError("exactly one regular vize archive member required")
        member = members[0]
        if not 64 <= member.size <= 256 * 1024 * 1024 or not member.mode & 0o111:
            raise ValueError("bounded executable CLI member required")
        source = stream.extractfile(member)
        data = source.read(member.size + 1)
        if len(data) != member.size:
            raise ValueError("complete declared CLI member required")
        if data[:6] != b"\x7fELF\x02\x01" or data[18:20] != b"\x3e\x00":
            raise ValueError("actual Linux x86_64 ELF member required")
    destination.mkdir(mode=0o700)
    binary = destination / "vize"
    with binary.open("xb") as output:
        output.write(data)
    binary.chmod(member.mode & 0o777)
    return {"member": member.name, "bytes": len(data), "mode": member.mode & 0o777,
            "sha256": hashlib.sha256(data).hexdigest(), "binary": str(binary)}


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: archive.py ARCHIVE FRESH_DESTINATION")
    print(json.dumps(extract(sys.argv[1], sys.argv[2])))
