"""Bind a real Linux x86_64 core to its observed physical executable."""
import json
from pathlib import Path
import struct


def core_pid(core):
    with core.open("rb") as stream:
        header = stream.read(64)
        if len(header) != 64 or header[:6] != b"\x7fELF\x02\x01":
            raise ValueError("Unsupported core ELF identity")
        if struct.unpack_from("<HH", header, 16) != (4, 62):
            raise ValueError("Expected Linux x86_64 ET_CORE")
        offset = struct.unpack_from("<Q", header, 32)[0]
        size, count = struct.unpack_from("<HH", header, 54)
        if size != 56 or not 0 < count <= 4096:
            raise ValueError("Invalid core program-header authority")
        for index in range(count):
            stream.seek(offset + index * size)
            program = stream.read(size)
            if len(program) != size:
                raise ValueError("Truncated core program header")
            if struct.unpack_from("<I", program)[0] != 4:
                continue
            note_offset = struct.unpack_from("<Q", program, 8)[0]
            note_size = struct.unpack_from("<Q", program, 32)[0]
            if note_size > 1048576:
                raise ValueError("Core notes exceed bounded identity parser")
            stream.seek(note_offset)
            notes = stream.read(note_size)
            if len(notes) != note_size:
                raise ValueError("Truncated core notes")
            cursor = 0
            while cursor < len(notes):
                if cursor + 12 > len(notes):
                    raise ValueError("Truncated core note header")
                name_size, descriptor_size, kind = struct.unpack_from("<III", notes, cursor)
                name_start = cursor + 12
                descriptor_start = name_start + ((name_size + 3) & ~3)
                end = descriptor_start + descriptor_size
                if end > len(notes):
                    raise ValueError("Truncated core note descriptor")
                if kind == 3 and notes[name_start:name_start + name_size] == b"CORE\0":
                    if descriptor_size < 28:
                        raise ValueError("Truncated Linux PRPSINFO")
                    return struct.unpack_from("<i", notes, descriptor_start + 24)[0]
                cursor = descriptor_start + ((descriptor_size + 3) & ~3)
    raise ValueError("Core has no authenticated Linux PRPSINFO process PID")


def producer(core, output, hash_file):
    pid = core_pid(core)
    if core.name != "core." + str(pid):
        raise ValueError("Core filename does not match actual producer PID")
    packets = [json.loads(path.read_text()) for path in (output / "processes").glob(str(pid) + "-*.json")]
    if not packets or len({packet["startTicks"] for packet in packets}) != 1:
        raise ValueError("Missing or ambiguous retained producer start ticks")
    identities = []
    for packet in packets:
        for item in packet["providers"]:
            if item["path"] == packet["executable"] and "sha256" in item:
                identities.append(dict(item, startTicks=packet["startTicks"], pid=pid))
    if not identities or any(item != identities[0] for item in identities):
        raise ValueError("Missing or changed observed producer executable")
    identity = identities[0]
    path = Path(identity["path"])
    before = path.stat()
    with path.open("rb") as stream:
        header = stream.read(64)
    if len(header) != 64 or header[:6] != b"\x7fELF\x02\x01" or struct.unpack_from("<HH", header, 16) not in [(2, 62), (3, 62)]:
        raise ValueError("Producer is not an authenticated x86_64 executable ELF")
    if before.st_ino != identity["mappedInode"] or before.st_size != identity["bytes"]:
        raise ValueError("Physical producer inode or size changed")
    if hash_file(path) != identity["sha256"]:
        raise ValueError("Physical producer executable hash changed")
    after = path.stat()
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
        raise ValueError("Producer executable changed during authentication")
    return identity
