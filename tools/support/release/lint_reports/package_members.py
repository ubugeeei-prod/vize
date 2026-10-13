"""Compare every installed regular member to an authenticated registry archive."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import sys
import tarfile

archive, directory = map(Path, sys.argv[1:])
root = directory.resolve(strict=True)
records = []
seen = set()
with tarfile.open(archive, "r:gz") as package:
    for member in package:
        name = PurePosixPath(member.name)
        assert name.parts[0] == "package" and ".." not in name.parts
        assert not name.is_absolute()
        if member.isdir():
            continue
        assert member.isfile(), "registry links and special members refused"
        relative = str(PurePosixPath(*name.parts[1:]))
        assert relative and relative not in seen
        seen.add(relative)
        target = root / relative
        assert target.is_file() and not target.is_symlink()
        assert target.resolve().is_relative_to(root)
        data = package.extractfile(member).read()
        assert target.read_bytes() == data, f"installed package differs: {relative}"
        records.append({"path": relative, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
assert records
actual = {str(file.relative_to(root)) for file in root.rglob("*") if file.is_file() and "node_modules" not in file.relative_to(root).parts}
assert actual == seen, "extra installed package members refused"
print(json.dumps(records, sort_keys=True))
