"""Bounded child-process probes of the real content-mapper worker stack.

RUST_MIN_STACK controls inherited Rust worker stacks before an explicit worker
budget is selected. Every trial retains its whole wire input, stdout, stderr,
and resource report, including aborted and timed-out children.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import time


DEPTHS = (128, 512, 1100, 2048, 4094)
STACKS = tuple(256 * 1024 * 2**power for power in range(10))
TIMEOUT_SECONDS = 30
OUTPUT_LIMIT_BYTES = 512 * 1024 * 1024
RSS_LIMIT_BYTES = 2 * 1024 * 1024 * 1024


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def witness(shape, depth):
    source = (
        '<script setup lang="ts">\n'
        'import Child from "./Child.vue";\n'
        "const rows = [1];\n"
        "</script>\n<template>\n"
    )
    if shape == "v-for":
        source += "".join(
            f'<template v-for="row{index} in rows">\n' for index in range(depth)
        )
        source += f'<Child :value="row{depth - 1}" />\n'
        source += "</template>\n" * depth
    else:
        source += "".join(
            f'<Child v-slot="slot{index}">\n' for index in range(depth)
        )
        source += f'<Child :value="slot{depth - 1}.value" />\n'
        source += "</Child>\n" * depth
    return source + "</template>\n"


def frames(source):
    requests = (
        {"id": 1, "method": "initialize", "params": {"positionEncodings": ["utf-8"]}},
        {
            "id": 2,
            "method": "openProject",
            "params": {"configFileName": "", "projectHandle": "stack"},
        },
        {
            "id": 3,
            "method": "transform",
            "params": {
                "fileName": "/project/Stack.vue",
                "content": source,
                "projectHandle": "stack",
            },
        },
    )
    output = bytearray()
    for request in requests:
        body = json.dumps({"jsonrpc": "2.0", **request}, separators=(",", ":")).encode()
        output.extend(f"Content-Length: {len(body)}\r\n\r\n".encode())
        output.extend(body)
    return output


def check_response(path, shape, depth):
    with path.open("rb") as stream:
        for expected_id in (1, 2, 3):
            header = stream.readline()
            assert header.startswith(b"Content-Length: "), header
            length = int(header.removeprefix(b"Content-Length: ").strip())
            assert 0 < length <= OUTPUT_LIMIT_BYTES, length
            assert stream.readline() == b"\r\n"
            body = stream.read(length)
            assert len(body) == length, (len(body), length)
            response = json.loads(body)
            assert response["jsonrpc"] == "2.0", response
            assert response["id"] == expected_id, response
            assert "error" not in response, response
            if expected_id == 3:
                text = response["result"]["text"]
                marker = f"// Component props in {shape} scope: "
                assert text.count(marker) == depth, (shape, depth, text.count(marker))
                deepest = f"row{depth - 1}" if shape == "v-for" else f"slot{depth - 1}.value"
                assert deepest in text, deepest
                text_hash = hashlib.sha256(text.encode()).hexdigest()
        assert stream.read(1) == b"", "unexpected trailing response bytes"
    return text_hash


def child_limits(timeout_seconds):
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_FSIZE, (OUTPUT_LIMIT_BYTES, OUTPUT_LIMIT_BYTES))
    resource.setrlimit(resource.RLIMIT_CPU, (timeout_seconds, timeout_seconds + 1))


def process_tree_rss(pid):
    pending = [pid]
    total = 0
    while pending:
        current = pending.pop()
        try:
            process = Path(f"/proc/{current}")
            total += int((process / "statm").read_text().split()[1]) * os.sysconf("SC_PAGE_SIZE")
            pending.extend(
                int(child) for child in (process / f"task/{current}/children").read_text().split()
            )
        except (FileNotFoundError, ProcessLookupError):
            pass
    return total


def trial(binary, directory, shape, depth, stack, timeout_seconds=TIMEOUT_SECONDS):
    directory.mkdir(parents=True)
    wire = frames(witness(shape, depth))
    (directory / "stdin.wire").write_bytes(wire)
    started = time.monotonic()
    timed_out = False
    rss_limit_exceeded = False
    peak_rss = 0
    with (directory / "stdin.wire").open("rb") as stdin, (
        directory / "stdout.wire"
    ).open("wb") as stdout, (directory / "stderr.txt").open("wb") as stderr:
        child = subprocess.Popen(
            ["/usr/bin/time", "-v", "-o", str(directory / "resources.txt"), str(binary), "content-mapper"],
            stdin=stdin,
            stdout=stdout,
            stderr=stderr,
            env={**os.environ, "RUST_MIN_STACK": str(stack)},
            start_new_session=True,
            preexec_fn=lambda: child_limits(timeout_seconds),
        )
        while child.poll() is None:
            peak_rss = max(peak_rss, process_tree_rss(child.pid))
            timed_out = time.monotonic() - started >= timeout_seconds
            rss_limit_exceeded = peak_rss > RSS_LIMIT_BYTES
            if timed_out or rss_limit_exceeded:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                break
            time.sleep(0.05)
        child.wait()
    result = {
        "shape": shape,
        "depth": depth,
        "inheritedMinStackBytes": stack,
        "returncode": child.returncode,
        "timedOut": timed_out,
        "timeoutSeconds": timeout_seconds,
        "rssLimitExceeded": rss_limit_exceeded,
        "peakProcessTreeRssBytes": peak_rss,
        "stackOverflow": b"stack overflow" in (directory / "stderr.txt").read_bytes(),
        "stdinTransport": "file",
        "wallSeconds": time.monotonic() - started,
        "stdinSha256": sha256(directory / "stdin.wire"),
        "stdoutSha256": sha256(directory / "stdout.wire"),
        "stderrSha256": sha256(directory / "stderr.txt"),
        "stdoutBytes": (directory / "stdout.wire").stat().st_size,
        "accepted": False,
    }
    if child.returncode == 0:
        try:
            result["virtualTextSha256"] = check_response(directory / "stdout.wire", shape, depth)
            assert (directory / "stderr.txt").read_bytes() == b"", "successful mapper wrote stderr"
            result["accepted"] = True
        except (AssertionError, ValueError, KeyError, TypeError) as error:
            result["validationError"] = str(error)
    (directory / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result), flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--profile", choices=("ci", "ci-opt"), required=True)
    parser.add_argument("--remaining-v-slot-only", action="store_true")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = {
        "sourceSha": args.source_sha,
        "binarySha256": sha256(binary),
        "profile": args.profile,
        "depths": DEPTHS,
        "candidateInheritedStacks": STACKS,
        "timeoutSeconds": TIMEOUT_SECONDS,
        "perFileOutputLimitBytes": OUTPUT_LIMIT_BYTES,
        "processTreeRssLimitBytes": RSS_LIMIT_BYTES,
        "remainingVSlotOnly": args.remaining_v_slot_only,
        "trials": [],
    }
    try:
        if args.remaining_v_slot_only:
            assert args.profile == "ci-opt", "focused follow-up uses the optimized source binary"
            accepted = []
            for stack in (32 * 1024**2, 64 * 1024**2, 128 * 1024**2):
                result = trial(binary, out / f"v-slot-4094-{stack}", "v-slot", 4094, stack, 90)
                report["trials"].append(result)
                assert not result["rssLimitExceeded"], "bounded RSS limit reached; stop the probe"
                if result["accepted"]:
                    if accepted:
                        assert result["stdoutSha256"] == accepted[0]["stdoutSha256"], "stack changed complete response"
                    accepted.append(result)
                elif not result["timedOut"]:
                    assert result["stackOverflow"], "non-stack failure is not a stack-budget observation"
            assert len(accepted) >= 2, "require complete output plus an independent larger-stack control"
            report["confirmedInheritedStacks"] = [trial["inheritedMinStackBytes"] for trial in accepted]
            report["complete"] = True
            return
        for shape in ("v-for", "v-slot"):
            candidate = 0
            for depth in DEPTHS:
                while candidate < len(STACKS):
                    stack = STACKS[candidate]
                    result = trial(binary, out / f"{shape}-{depth}-{stack}", shape, depth, stack)
                    report["trials"].append(result)
                    if result["accepted"]:
                        break
                    assert not result["timedOut"], "timeout is not a stack-budget observation"
                    assert not result["rssLimitExceeded"], "bounded RSS limit reached; stop the probe"
                    assert result["returncode"] != 0, "semantic refusal is not a stack-budget observation"
                    assert result["stackOverflow"], "non-stack failure is not a stack-budget observation"
                    candidate += 1
                assert candidate < len(STACKS), (shape, depth, "no candidate stack completed")
                control_stack = STACKS[min(candidate + 1, len(STACKS) - 1)]
                control = trial(binary, out / f"{shape}-{depth}-control", shape, depth, control_stack)
                report["trials"].append(control)
                assert control["accepted"], "larger stack control must complete"
                assert control["stdoutSha256"] == result["stdoutSha256"], "stack changed complete response"
        report["complete"] = True
    finally:
        (out / "report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
