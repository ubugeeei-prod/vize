"""Bounded Linux RSS observations; no environment/argv inspection or peak/PSS claim."""
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import sys
import time


def process(pid):
    stat = Path(f"/proc/{pid}/stat").read_text()
    fields = stat[stat.rindex(")") + 2:].split()
    return {"pid": pid, "parentPid": int(fields[1]), "startTicks": fields[19],
            "state": fields[0], "rssKiB": int(fields[21]) * os.sysconf("SC_PAGE_SIZE") // 1024}


def owned(table, root_pid, root_ticks, known):
    root = table.get(root_pid)
    if root and root["startTicks"] != root_ticks:
        raise ValueError("root PID was reused")
    selected = {pid for pid, ticks in known.items()
                if pid in table and table[pid]["startTicks"] == ticks}
    if root:
        selected.add(root_pid)
    while True:
        children = {pid for pid, row in table.items() if row["parentPid"] in selected}
        if children <= selected:
            break
        selected.update(children)
    return sorted(selected)


def main():
    root_pid, root_ticks, destination = int(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    known = {root_pid: root_ticks}
    executables = {}
    deadline = time.monotonic() + 330
    written = 0
    stopped = False
    cleanup_deadline = None
    failure = None
    with destination.open("x") as output:
        def emit(row):
            nonlocal written
            line = json.dumps(row, separators=(",", ":")) + "\n"
            written += len(line.encode())
            if written > 64 * 1024 * 1024:
                raise ValueError("RSS raw output quota exceeded")
            output.write(line)
            output.flush()

        emit({"kind": "start", "pid": root_pid, "startTicks": root_ticks,
              "intervalMs": 50, "clock": "CLOCK_MONOTONIC", "ns": str(time.monotonic_ns())})
        try:
            while True:
                start = time.monotonic()
                if start > deadline:
                    raise ValueError("RSS sampler exceeded finite deadline")
                table = {}
                for entry in Path("/proc").iterdir():
                    if not entry.name.isdecimal():
                        continue
                    pid = int(entry.name)
                    try:
                        table[pid] = process(pid)
                    except (FileNotFoundError, ProcessLookupError):
                        continue
                    except PermissionError:
                        if pid in known:
                            raise
                members = []
                for pid in owned(table, root_pid, root_ticks, known):
                    row = table[pid]
                    known[pid] = row["startTicks"]
                    if row["state"] == "Z":
                        continue
                    key = (pid, row["startTicks"])
                    try:
                        executable = os.readlink(f"/proc/{pid}/exe")
                        if key not in executables:
                            with open(f"/proc/{pid}/exe", "rb") as binary:
                                digest = hashlib.file_digest(binary, "sha256").hexdigest()
                            executables[key] = (executable, digest)
                        if executables[key][0] != executable:
                            raise ValueError("observed process executable changed")
                    except (FileNotFoundError, ProcessLookupError):
                        continue  # A short-lived process is explicitly unobserved, never RSS zero.
                    members.append({**row, "executable": executable,
                                    "executableSha256": executables[key][1]})
                emit({"kind": "sample", "ns": str(time.monotonic_ns()), "members": members,
                      "scanMs": (time.monotonic() - start) * 1000})
                if not stopped and select.select([sys.stdin], [], [], 0)[0]:
                    if sys.stdin.readline().strip() != "stop":
                        raise ValueError("explicit server-close stop signal is required")
                    stopped = True
                    cleanup_deadline = time.monotonic() + 1
                if stopped and not members:
                    break
                if stopped and time.monotonic() > cleanup_deadline:
                    raise ValueError("observed server descendants survived shutdown")
                time.sleep(max(0, 0.05 - (time.monotonic() - start)))
        except Exception as error:
            failure = str(error)
        survivors = []
        for pid, ticks in known.items():
            try:
                row = process(pid)
                if row["startTicks"] == ticks and row["state"] != "Z":
                    survivors.append({"pid": pid, "startTicks": ticks})
                    os.kill(pid, signal.SIGKILL)
            except (FileNotFoundError, ProcessLookupError):
                pass
        emit({"kind": "finish", "ns": str(time.monotonic_ns()), "stopped": stopped,
              "failure": failure, "survivorsBeforeForcedCleanup": survivors,
              "limits": "50ms sampled RSS sum; missed short-lived processes, not true peak or PSS"})
    if failure or survivors or not stopped:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
