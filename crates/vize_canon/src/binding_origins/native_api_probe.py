#!/usr/bin/env python3
"""Test-only exec carrier: no forwarding or replacement API implementation."""
import json
import os
import sys

backend = os.environ["VIZE_ORIGIN_API_BACKEND"]
output = os.environ["VIZE_ORIGIN_API_CAPTURE"]
argv = [backend, *sys.argv[1:]]
with open(os.path.join(output, "actual-launch.json"), "x") as stream:
    json.dump({"pid": os.getpid(), "argv": argv, "transport": "direct-stdio"}, stream)
os.execv(backend, argv)
