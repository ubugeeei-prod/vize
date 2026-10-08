#!/usr/bin/env python3
"""Observe the previous physical native owners at the actual launch boundary."""
import json
import os
from pathlib import Path
import sys

native, trace, *args = sys.argv[1:]
trace = Path(trace)
previous = [json.loads(line) for line in trace.read_text().splitlines()] if trace.exists() else []
live = []
for owner in previous:
    base = Path(f"/proc/{owner['pid']}")
    try:
        tail = (base / "stat").read_text().rsplit(") ", 1)[1].split()
        if tail[19] != owner["birth"]:
            continue
        if tail[0] == "Z":
            live.append({"pid": owner["pid"], "birth": owner["birth"], "role": owner["role"], "state": "Z"})
            continue
        physical = os.readlink(base / "exe")
        command = (base / "cmdline").read_bytes().split(b"\0")
    except FileNotFoundError:
        continue
    if physical != native or owner["role"].encode() not in command:
        raise RuntimeError(f"recorded native owner changed identity: {owner}, {physical}, {command}")
    live.append({"pid": owner["pid"], "birth": owner["birth"], "role": owner["role"], "state": tail[0]})

birth = Path(f"/proc/{os.getpid()}/stat").read_text().rsplit(") ", 1)[1].split()[19]
role = "--api" if "--api" in args else "--lsp"
record = {"pid": os.getpid(), "birth": birth, "role": role, "previousNativeOwners": live}
fd = os.open(trace, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
os.write(fd, (json.dumps(record) + "\n").encode())
os.close(fd)
os.execv(native, [native, *args])
