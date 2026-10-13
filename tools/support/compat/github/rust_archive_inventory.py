#!/usr/bin/env python3
"""Hash every decoded tar byte and record every logical member without extraction."""
import hashlib
import json
import subprocess
import tarfile
import time
from pathlib import Path

from artifact_payload_files import digest_file, json_bytes, require, safe_name


class HashedReader:
    def __init__(self, stream):
        self.stream = stream
        self.digest = hashlib.sha256()
        self.size = 0

    def read(self, size=-1):
        data = self.stream.read(size)
        self.digest.update(data)
        self.size += len(data)
        return data


def inventory(archive, destination, zstd="zstd"):
    started = time.monotonic()
    destination = Path(destination)
    require(not destination.exists(), "inventory destination must be fresh")
    members = []
    with destination.with_suffix(".stderr").open("xb") as errors:
        decoder = subprocess.Popen([zstd, "-d", "-c", str(archive)], stdout=subprocess.PIPE,
                                   stderr=errors)
        stream = HashedReader(decoder.stdout)
        try:
            # Continue past zero records: capture appended members as well as padding.
            with tarfile.open(fileobj=stream, mode="r|", ignore_zeros=True) as tar:
                for member in tar:
                    name = safe_name(member.name.rstrip("/"))
                    require(name.split("/")[0] == "target", "tar member lacks target prefix")
                    require(member.isreg() or member.isdir() or member.issym() or member.islnk(),
                            "unsupported tar member type")
                    body = hashlib.sha256()
                    body_size = 0
                    if member.isreg():
                        with tar.extractfile(member) as file:
                            while chunk := file.read(1024 * 1024):
                                body.update(chunk)
                                body_size += len(chunk)
                        require(body_size == member.size, "tar member body is incomplete")
                    members.append({
                        "path": name, "type": member.type.decode("ascii"),
                        "mode": member.mode, "size": member.size,
                        "bodySha256": body.hexdigest() if member.isreg() else None,
                        "uid": member.uid, "gid": member.gid, "mtime": member.mtime,
                        "uname": member.uname, "gname": member.gname,
                        "linkname": member.linkname, "paxHeaders": member.pax_headers,
                    })
            while stream.read(1024 * 1024):
                pass
            require(decoder.wait() == 0, "complete zstd decoding failed")
        finally:
            decoder.stdout.close()
            if decoder.poll() is None:
                decoder.kill()
                decoder.wait()
    require(members, "tar inventory is empty")
    report = {"schemaVersion": 1, "tarSha256": stream.digest.hexdigest(),
              "tarBytes": stream.size, "members": members}
    destination.write_bytes(json_bytes(report))
    summary = {"archiveSha256": digest_file(archive), "archiveBytes": Path(archive).stat().st_size,
               "inventorySha256": digest_file(destination), "memberCount": len(members),
               "tarSha256": report["tarSha256"], "tarBytes": stream.size,
               "decodeAndInventoryMs": round((time.monotonic() - started) * 1000)}
    destination.with_suffix(".summary.json").write_bytes(json_bytes(summary))
    return report, summary


if __name__ == "__main__":
    import sys
    _, result = inventory(sys.argv[1], sys.argv[2])
    print(json.dumps(result, sort_keys=True))
