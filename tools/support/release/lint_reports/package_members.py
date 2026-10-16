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
# npm archives may own bundled node_modules members. Keep unrelated installed
# dependencies outside this package's archive scope, but include every physical
# file under a bundled package root so injected bundle files cannot disappear.
bundles = set()
for name in seen:
    parts = PurePosixPath(name).parts
    if "node_modules" in parts:
        index = parts.index("node_modules") + 1
        count = 2 if parts[index].startswith("@") else 1
        bundles.add(PurePosixPath(*parts[:index + count]))
actual = set()
for file in root.rglob("*"):
    if not file.is_file():
        continue
    relative = file.relative_to(root)
    if "node_modules" not in relative.parts or any(relative.is_relative_to(bundle) for bundle in bundles):
        assert not file.is_symlink(), "installed package links refused"
        actual.add(str(relative))
assert actual == seen, "extra installed package members refused"
print(json.dumps(records, sort_keys=True))
