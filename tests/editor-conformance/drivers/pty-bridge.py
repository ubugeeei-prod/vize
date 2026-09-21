#!/usr/bin/env python3
"""Run a terminal editor in a pseudo-terminal for the TS-45 Helix driver.

Keystrokes arrive on this process's stdin (a pipe from the Node driver) and
are written to the terminal; everything the editor draws is appended to the
--screen file so a failing CI run can show what the editor displayed. The
editor's startup terminal queries get the answers a plain VT220-class
terminal gives (a DA1 reply and nothing else), so it never waits on them.

usage: pty-bridge.py --cols N --rows N --screen FILE -- COMMAND [ARG ...]
"""

import argparse
import errno
import fcntl
import os
import pty
import select
import struct
import sys
import termios

# Primary device attributes: "VT220 with ANSI color". Editors send DA1 after
# their feature probes and treat the reply as "no more answers are coming".
DA1_QUERY = b"\x1b[c"
DA1_REPLY = b"\x1b[?62;22c"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cols", type=int, required=True)
    parser.add_argument("--rows", type=int, required=True)
    parser.add_argument("--screen", required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("missing COMMAND")

    child_pid, terminal = pty.fork()
    if child_pid == 0:
        size = struct.pack("HHHH", args.rows, args.cols, 0, 0)
        fcntl.ioctl(sys.stdin.fileno(), termios.TIOCSWINSZ, size)
        # A shell is the session leader, as in a real terminal: when the
        # editor itself led the session, its exit hung up the terminal and
        # SIGHUP killed the language server it had just asked to `exit`
        # before the server could. The shell outlives the editor by a grace
        # period, the way a terminal stays open after the editor quits.
        os.execvp(
            "/bin/sh",
            ["/bin/sh", "-c", '"$@"; status=$?; sleep 5; exit "$status"', "sh", *command],
        )

    screen = open(args.screen, "ab", buffering=0)
    keys = sys.stdin.fileno()
    tail = b""
    open_inputs = [terminal, keys]
    while True:
        readable, _, _ = select.select(open_inputs, [], [], 0.05)
        if terminal in readable:
            try:
                chunk = os.read(terminal, 65536)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
                chunk = b""
            if not chunk:
                break
            screen.write(chunk)
            window = tail + chunk
            for _ in range(window.count(DA1_QUERY)):
                os.write(terminal, DA1_REPLY)
            tail = window[-(len(DA1_QUERY) - 1):]
        if keys in readable:
            data = os.read(keys, 4096)
            if data:
                os.write(terminal, data)
            else:
                open_inputs.remove(keys)
        waited, status = os.waitpid(child_pid, os.WNOHANG)
        if waited == child_pid:
            return os.waitstatus_to_exitcode(status)
    _, status = os.waitpid(child_pid, 0)
    return os.waitstatus_to_exitcode(status)


if __name__ == "__main__":
    raise SystemExit(main())
