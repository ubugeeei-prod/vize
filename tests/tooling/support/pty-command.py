#!/usr/bin/env python3
"""Run a command in a pseudo-terminal and answer after a prompt appears."""

import errno
import json
import os
import pty
import select
import signal
import sys
import time


class LifecycleEvidence:
    """Bounded Linux process metadata; never record argv, input, or environment."""

    def __init__(self, child_pid: int, terminal_fd: int):
        self.child_pid = child_pid
        self.terminal_fd = terminal_fd
        self.started = time.monotonic()
        self.next_snapshot = 0.0
        self.records = 0
        self.destination = None
        evidence_path = os.environ.get("VIZE_PTY_EVIDENCE")
        if evidence_path:
            self.destination = open(f"{evidence_path}.{os.getpid()}.jsonl", "x")

    def record(self, event: str, force: bool = False, failure: bool = False) -> None:
        now = time.monotonic()
        if (not self.destination and not failure) or self.records >= 128:
            return
        if not force and now < self.next_snapshot:
            return
        self.next_snapshot = now + 0.5
        processes = []
        if sys.platform == "linux":
            for entry in os.scandir("/proc"):
                if not entry.name.isdecimal():
                    continue
                try:
                    with open(f"{entry.path}/stat") as source:
                        stat = source.read().rsplit(")", 1)[1].split()
                    if int(stat[3]) != self.child_pid:
                        continue
                    with open(f"{entry.path}/cmdline", "rb") as source:
                        arguments = source.read(4096).split(b"\0")
                    binary = os.path.basename(arguments[0]).decode(errors="replace")
                    known = {"vp", "node", "sh", "bash", "moon", "moonc", "moondoc", "gcc", "cc", "clang", "git", "release.exe", "release"}
                    phase = binary if binary in known else "other"
                    if binary == "moon":
                        for operation in (b"update", b"run", b"build"):
                            if operation in arguments:
                                phase += ":" + operation.decode()
                                break
                    with open(f"{entry.path}/wchan") as source:
                        waiting = source.read().strip()
                    processes.append({"pid": int(entry.name), "ppid": int(stat[1]),
                                      "pgrp": int(stat[2]), "session": int(stat[3]),
                                      "foreground": int(stat[5]), "state": stat[0],
                                      "phase": phase, "wait": waiting})
                except (OSError, ValueError, IndexError):
                    continue
                if len(processes) >= 256:
                    break
        try:
            foreground = os.tcgetpgrp(self.terminal_fd)
        except OSError:
            foreground = None
        record = {"event": event, "elapsed": round(now - self.started, 3),
                  "child": self.child_pid, "foreground": foreground,
                  "processes": sorted(processes, key=lambda process: process["pid"])}
        if self.destination:
            self.destination.write(json.dumps(record) + "\n")
            self.destination.flush()
        if failure:
            sys.stderr.write("PTY lifecycle: " + json.dumps(record) + "\n")
            sys.stderr.flush()
        self.records += 1

    def close(self) -> None:
        if self.destination:
            self.destination.close()


def wait_for_child(child_pid: int, timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            waited_pid, _ = os.waitpid(child_pid, os.WNOHANG)
        except ChildProcessError:
            return True
        if waited_pid == child_pid:
            return True
        time.sleep(0.05)
    return False


def terminate_process_group(child_pid: int) -> None:
    try:
        os.killpg(child_pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    wait_for_child(child_pid, 1)
    try:
        os.killpg(child_pid, 0)
    except ProcessLookupError:
        return
    try:
        os.killpg(child_pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    wait_for_child(child_pid, 1)


def main() -> int:
    if len(sys.argv) < 4:
        raise SystemExit("usage: pty-command.py PROMPT RESPONSE COMMAND [ARG ...]")

    prompt = sys.argv[1].encode()
    response = sys.argv[2].encode()
    command = sys.argv[3:]
    child_pid, terminal_fd = pty.fork()
    if child_pid == 0:
        os.execvp(command[0], command)

    output = bytearray()
    answered = False
    deadline = time.monotonic() + 25
    evidence = LifecycleEvidence(child_pid, terminal_fd)
    status = None
    while status is None:
        if time.monotonic() >= deadline:
            evidence.record("deadline", force=True, failure=True)
            terminate_process_group(child_pid)
            evidence.record("terminated", force=True)
            evidence.close()
            os.close(terminal_fd)
            return 124

        evidence.record("waiting")

        readable, _, _ = select.select([terminal_fd], [], [], 0.1)
        if readable:
            try:
                chunk = os.read(terminal_fd, 4096)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                chunk = b""
            if chunk:
                os.write(sys.stdout.fileno(), chunk)
                output.extend(chunk)
                if not answered and prompt in output:
                    evidence.record("prompt", force=True)
                    os.write(terminal_fd, response)
                    answered = True
                    evidence.record("answered", force=True)

        waited_pid, child_status = os.waitpid(child_pid, os.WNOHANG)
        if waited_pid == child_pid:
            status = child_status

    evidence.record("child-exit", force=True)
    evidence.close()
    os.close(terminal_fd)
    if not answered:
        return 125
    return os.waitstatus_to_exitcode(status)


if __name__ == "__main__":
    raise SystemExit(main())
