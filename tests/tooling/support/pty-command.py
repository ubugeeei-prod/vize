#!/usr/bin/env python3
"""Run a command in a pseudo-terminal and answer after a prompt appears."""

import errno
import os
import pty
import select
import signal
import sys
import time


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
    status = None
    while status is None:
        if time.monotonic() >= deadline:
            terminate_process_group(child_pid)
            os.close(terminal_fd)
            return 124

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
                    os.write(terminal_fd, response)
                    answered = True

        waited_pid, child_status = os.waitpid(child_pid, os.WNOHANG)
        if waited_pid == child_pid:
            status = child_status

    os.close(terminal_fd)
    if not answered:
        return 125
    return os.waitstatus_to_exitcode(status)


if __name__ == "__main__":
    raise SystemExit(main())
