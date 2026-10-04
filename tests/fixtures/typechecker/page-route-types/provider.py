"""Stage the exact official Router provider without installing dependencies."""

import base64
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import sys
import tarfile
import urllib.request

URL = "https://registry.npmjs.org/vue-router/-/vue-router-5.1.0.tgz"
INTEGRITY = "HAbiLzLEHQwxPgvsbOJDAwtavszEgLwri6XfyrsPECIFez8+59xc9LofWVdc/HEaSRT822lJ8H9Ns38VVond5g=="
SOURCE = "c0e3226dabccd7596b996ce851386997ea2d3cca"
MAX_BYTES = 2_000_000


def members(raw, integrity, count, total):
    """Validate the whole archive before any member is written."""
    if len(raw) > MAX_BYTES or base64.b64encode(hashlib.sha512(raw).digest()).decode() != integrity:
        raise ValueError("provider archive integrity or size mismatch")
    files = []
    names = set()
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as archive:
        entries = archive.getmembers()
        if len(entries) != count or sum(entry.size for entry in entries) != total:
            raise ValueError("provider archive file count or unpacked size mismatch")
        for entry in entries:
            path = PurePosixPath(entry.name)
            if (
                not entry.isfile()
                or entry.linkname
                or path.is_absolute()
                or len(path.parts) < 2
                or path.parts[0] != "package"
                or any(part in ("", ".", "..") for part in entry.name.split("/"))
                or "\\" in entry.name
                or "\x00" in entry.name
                or entry.name in names
                or any(entry.name.startswith(name + "/") or name.startswith(entry.name + "/") for name in names)
            ):
                raise ValueError("unsafe or duplicate provider archive member")
            names.add(entry.name)
            body = archive.extractfile(entry).read()
            if len(body) != entry.size:
                raise ValueError("provider member length mismatch")
            files.append((str(path.relative_to("package")), body, entry.mode))
    return files


def validate_package(files):
    table = {name: body for name, body, _mode in files}
    package = json.loads(table["package.json"])
    if package.get("name") != "vue-router" or package.get("version") != "5.1.0":
        raise ValueError("unexpected provider identity")
    exports = package["exports"]
    required = [
        exports["."]["types"],
        exports["./experimental"]["types"],
        exports["./auto-routes"]["types"],
        exports["./auto"]["types"],
        exports["./volar/sfc-typed-router"]["default"],
        exports["./volar/sfc-typed-router"]["types"],
        "./LICENSE",
    ]
    if any(not target.startswith("./") or target[2:] not in table for target in required):
        raise ValueError("missing provider declaration, plugin or license")
    return package


def stage(destination):
    if destination.exists() or destination.is_symlink() or not destination.parent.is_dir():
        raise ValueError("provider destination must be a new isolated directory")
    with urllib.request.urlopen(URL, timeout=30) as response:
        if response.status != 200 or response.geturl() != URL:
            raise ValueError("unexpected official provider response")
        raw = response.read(MAX_BYTES + 1)
    files = members(raw, INTEGRITY, 70, 1_167_017)
    package = validate_package(files)
    destination.mkdir()
    package_root = destination / "package"
    rows = []
    for name, body, mode in files:
        path = package_root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as output:
            output.write(body)
        path.chmod(mode & 0o777)
        rows.append({"path": name, "bytes": len(body), "sha256": hashlib.sha256(body).hexdigest(), "mode": mode})
    (destination / "vue-router-5.1.0.tgz").write_bytes(raw)
    receipt = {
        "schema": "vize.router-page-provider",
        "url": URL,
        "integrity": "sha512-" + INTEGRITY,
        "archiveBytes": len(raw),
        "archiveSha256": hashlib.sha256(raw).hexdigest(),
        "publishedSource": SOURCE,
        "pageAndMapSource": "feed382f2fbfe38b3892ea780f5aea3d5459986a",
        "typeofReferenceSource": "071f1969be1348e797a55d0d8afb72b8068154dc",
        "files": rows,
        "exports": package["exports"],
    }
    (destination / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: provider.py NEW_ISOLATED_DESTINATION")
    stage(Path(sys.argv[1]))
