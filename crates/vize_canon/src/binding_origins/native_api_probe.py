#!/usr/bin/env python3
"""Test-only exec carrier: no forwarding or replacement API implementation."""
import ctypes
import json
import os
import signal
import sys

parent = os.getppid()
if ctypes.CDLL(None, use_errno=True).prctl(1, signal.SIGKILL, 0, 0, 0) != 0:
    raise OSError(ctypes.get_errno(), "cannot install parent-death cleanup")
if os.getppid() != parent:
    raise RuntimeError("native owner disappeared during setup")

backend = os.environ["VIZE_ORIGIN_API_BACKEND"]
output = os.environ["VIZE_ORIGIN_API_CAPTURE"]
argv = [backend, *sys.argv[1:]]
with open(os.path.join(output, "actual-launch.json"), "x") as stream:
    json.dump({"pid": os.getpid(), "argv": argv, "transport": "direct-stdio"}, stream)
os.execv(backend, argv)
