"""Diagnostic saturation; every original input/child outcome remains strict."""
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

capsule = Path(__file__).with_name("mounted-child-capsule")
original = (capsule / "input.json").read_bytes()
assert hashlib.sha256(original).hexdigest() == "6419fea8a049f0c000d0a637380d2e601160dc404a3a758ae5be2f27b78b6722"
reference = json.loads((capsule / "stdout.json").read_bytes())
fixtures = [json.loads(line) for line in Path("tests/formal/impeto/fixtures/ivm-matrix.behavior.jsonl").read_text().splitlines()]
original_fixture = [fixture for fixture in fixtures if fixture["name"] == "object-positional-toggle-open"]
assert len(original_fixture) == 1 and reference == original_fixture[0]["trace"]
directory = Path(os.environ["VIZE_MOUNTED_CHILD_CUSTODY"]) / "saturation"
directory.mkdir()
runner = Path.cwd() / "tests/tooling/support/davinci-mounted-trace.mjs"
stopped = threading.Event()
environment = dict(os.environ, VIZE_MOUNTED_CHILD_ROLE="original-input-saturation")

def observe(index):
    if stopped.is_set():
        return None
    case = directory / str(index)
    case.mkdir()
    (case / "stdin.bin").write_bytes(original)
    started = time.monotonic()
    child = subprocess.Popen(["node", str(runner)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment)
    stdout, stderr = child.communicate(original)
    (case / "stdout.bin").write_bytes(stdout)
    (case / "stderr.bin").write_bytes(stderr)
    try:
        equal = json.loads(stdout) == reference
    except (ValueError, UnicodeError):
        equal = False
    receipt = {"index": index, "pid": child.pid, "returncode": child.returncode, "signal": -child.returncode if child.returncode < 0 else None, "referenceEqual": equal, "seconds": time.monotonic() - started}
    (case / "receipt.json").write_text(json.dumps(receipt) + "\n")
    if child.returncode != 0 or not equal:
        stopped.set()
    return receipt

with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
    receipts = [receipt for receipt in executor.map(observe, range(512)) if receipt is not None]
failures = [receipt for receipt in receipts if receipt["returncode"] != 0 or not receipt["referenceEqual"]]
summary = {"planned": 512, "observed": len(receipts), "parallel": 4, "failures": failures, "sourceSha": os.environ["VIZE_MOUNTED_CHILD_SOURCE_SHA"], "inputSha256": hashlib.sha256(original).hexdigest()}
(directory / "summary.json").write_text(json.dumps(summary) + "\n")
print(json.dumps(summary), flush=True)
sys.exit(1 if failures or len(receipts) != 512 else 0)
