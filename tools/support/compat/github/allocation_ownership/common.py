"""Frozen inputs and transport for the authorized #7764 diagnostic only."""

import hashlib
import json
import subprocess
from pathlib import Path

REPO = "ubugeeei-prod/vize"
BRANCH = "ci/vapor-allocation-ownership-20261004"
CASE = "native_lane_stays_within_its_allocation_ceilings"
BINARY = "davinci_vapor_native_budget"
POSITIVE_CASE = "alloc::tests::scripted_sequence_has_exact_counter_deltas"
ATTEMPTS = 8
FEATURE = "allocation-window-diagnostics"
FIXTURES = [("text_runs", 75), ("events", 74), ("expressions", 110),
            ("components", 100), ("templates", 134), ("spreads", 106),
            ("control_flow", 158)]
SOURCES = [
    {"label": "base", "sha": "b9be9065b872086726158b67d1b4e7b8dee78a14",
     "run": 37180545481, "archive_id": 11294942508,
     "archive_zip_sha256": "c9a43b2722809f24966f81814e0e470f94c8689801212ef7792ff0f467a48539",
     "timing_id": 11294418430,
     "timing_zip_sha256": "8aaddacae70ec3870556cd4cb3f3c5fa39304df29f8ac6d09d9a1ea091cd3cbb"},
    {"label": "candidate", "sha": "e1c0c8609e64035a9705cd76257e9d3fdaf966b4",
     "run": 37181078166, "archive_id": 11294949814,
     "archive_zip_sha256": "5dc11633be55cb47b38472524efb2305d5e721486bbf40a54fd6b23357e268f0",
     "timing_id": 11295795336,
     "timing_zip_sha256": "2c5ea477f75ffafb92ca0270f3d2a59ab7f6cbc1a646c446b2550c8f6f555e7f"},
]
ORIGINAL_BLOBS = {
    "tools/benchmarks/crates/davinci_harness/src/alloc.rs": "8434c67c26d0b95e56a2206557329dfdaa4b5fc0",
    "tools/benchmarks/crates/davinci_harness/Cargo.toml": "acf282eacb05189084ade7335bc42967f2375afa",
    "crates/vize_atelier_vapor/tests/davinci_vapor_native_budget.rs": "79a32dc7f6fa4c34662ac14bf489b1c97bc376ee",
}
OVERLAY = [*ORIGINAL_BLOBS,
    "tools/benchmarks/crates/davinci_harness/src/alloc/diagnostics.rs",
    "tools/benchmarks/crates/davinci_harness/src/alloc/diagnostics/enabled.rs",
    "tools/benchmarks/crates/davinci_harness/src/alloc/diagnostics/enabled/ledger.rs"]


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + "\n")


def command(args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def git(root, *args):
    return command(["git", *args], root)


def api(path):
    return json.loads(command(["gh", "api", f"repos/{REPO}/{path}"]))
