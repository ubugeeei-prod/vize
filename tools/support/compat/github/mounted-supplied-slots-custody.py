#!/usr/bin/env python3
"""One immutable supplied-slot diagnostic; failed outcomes are never waived."""
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import threading
import time

SOURCE = "f2ade1f7ca6dda81e14a75b46cb43980bd9762e2"
TREE = "6d4c9627d893bfeb9923cb116cf516241fbd2640"
RUNNER = "tests/tooling/support/davinci-mounted-trace.mjs"
HASHES = {
    "crates/vize_atelier_vapor/tests/davinci_mounted_behavior/runtime.rs": "cdd03c41ddabdeff064174a9c549c932104e648793484a4c9606432f5b6c0660",
    "crates/vize_atelier_vapor/tests/davinci_mounted_behavior/slots.rs": "fe01fc9cb0109db1d6d330be184e7e09df822f441c096894aa2e901ba6fe4686",
    RUNNER: "383e0745fb886abeb9e9a7dadd121f372fe6dd4eee3004af4ed2c4bd14127484",
    "tests/formal/impeto/fixtures/slot-reference.cases.jsonl": "755a250dd230e3e088b4506ae724ee44e50b3a0f34ae344caefd16f7d1a30db5",
    "tests/formal/impeto/fixtures/slot-reference.behavior.jsonl": "c533b48488489db8a755f8e7f0c83b8e62032d9057fc2cd36c9e459e0e172352",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    result = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def verify_source(root):
    actual = subprocess.check_output(["git", "rev-parse", "HEAD", "HEAD^{tree}"], cwd=root).decode().splitlines()
    assert actual == [SOURCE, TREE], "Wrong literal source/tree; no child may start"
    for name, expected in HASHES.items():
        assert digest((root / name).read_bytes()) == expected, f"Wrong source bytes: {name}"
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--"], cwd=root, check=True)
    return {"sha": SOURCE, "tree": TREE, "files": HASHES}


def forward(command, environment, directory, stdin, stdout, stderr):
    """Tee all bytes concurrently; wait the real child, never signal the launcher."""
    started = time.time_ns()
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, env=environment)
    errors = []

    def relay(reader, destination, name, close=False):
        enabled = True
        with (directory / name).open("wb") as retained:
            while True:
                data = reader.read1(65536) if hasattr(reader, "read1") else reader.read(65536)
                if not data:
                    break
                retained.write(data)
                if enabled:
                    try:
                        view = memoryview(data)
                        while view:
                            written = destination.write(view)
                            assert written and written <= len(view), "Incomplete pipe write"
                            view = view[written:]
                        destination.flush()
                    except (OSError, ValueError, AssertionError) as error:
                        errors.append({"stream": name, "error": repr(error)})
                        enabled = False
        if close:
            try:
                destination.close()
            except OSError as error:
                errors.append({"stream": name, "error": repr(error)})

    threads = [threading.Thread(target=relay, args=args) for args in [
        (stdin, child.stdin, "stdin.bin", True),
        (child.stdout, stdout, "stdout.bin"), (child.stderr, stderr, "stderr.bin"),
    ]]
    for thread in threads:
        thread.start()
    status = child.wait()
    for thread in threads:
        thread.join()
    packet = {"pid": child.pid, "launcherPid": os.getpid(), "rustPid": os.getppid(),
              "startedNs": started, "endedNs": time.time_ns(), "returncode": status,
              "signal": -status if status < 0 else None, "relayErrors": errors,
              "raw": {name: {"bytes": (directory / name).stat().st_size,
                             "sha256": digest((directory / name).read_bytes())}
                      for name in ["stdin.bin", "stdout.bin", "stderr.bin"]}}
    write_json(directory / "wait.json", packet)
    return packet


def launch():
    root = Path(os.environ["GITHUB_WORKSPACE"]).resolve()
    real_node = os.environ["VIZE_SUPPLIED_SLOTS_NODE"]
    arguments = sys.argv[1:]
    if len(arguments) != 1 or Path(arguments[0]).resolve() != root / RUNNER:
        os.execv(real_node, [real_node, *arguments])
    authority = verify_source(root)
    custody = Path(os.environ["VIZE_MOUNTED_CHILD_CUSTODY"])
    directory = custody / "children" / str(os.getpid())
    directory.mkdir(parents=True)
    tools = Path(__file__).resolve().parent
    env = dict(os.environ, VIZE_SUPPLIED_SLOTS_PACKET=str(directory),
               VIZE_MOUNTED_CHILD_SOURCE_SHA=SOURCE,
               VIZE_MOUNTED_CHILD_ROLE="current-f2-supplied-slots",
               VIZE_MOUNTED_TRACE_OBSERVATIONS="1",
               VIZE_VUE_RUNTIME_TRACE_EVIDENCE=str(directory / "trace.jsonl"))
    env["NODE_OPTIONS"] += f" --require={tools / 'mounted-supplied-slots-custody.cjs'}"
    write_json(directory / "source.json", authority)
    packet = forward([real_node, *arguments], env, directory,
                     sys.stdin.buffer, sys.stdout.buffer, sys.stderr.buffer)
    # The actual wait signal stays in wait.json. Exit139 is mediation, not a second core.
    sys.exit(1 if packet["relayErrors"] else 128 - packet["returncode"]
             if packet["returncode"] < 0 else packet["returncode"])


def elf_segments(file):
    file.seek(0)
    header = file.read(64)
    assert header[:6] == b"\x7fELF\x02\x01" and struct.unpack_from("<H", header, 18)[0] == 62
    offset = struct.unpack_from("<Q", header, 32)[0]
    size, count = struct.unpack_from("<HH", header, 54)
    segments = []
    for index in range(count):
        file.seek(offset + index * size)
        segments.append(struct.unpack("<IIQQQQQQ", file.read(56)))
    return segments


def elf_notes(file):
    notes = []
    for segment in elf_segments(file):
        if segment[0] != 4:
            continue
        file.seek(segment[2])
        data = file.read(segment[5])
        offset = 0
        while offset + 12 <= len(data):
            names, size, kind = struct.unpack_from("<III", data, offset)
            start = offset
            offset += 12
            owner = data[offset:offset + names].rstrip(b"\0")
            offset += (names + 3) & ~3
            value = data[offset:offset + size]
            offset += (size + 3) & ~3
            notes.append((owner, kind, value, segment[2] + start, data[start:offset]))
    return notes


def core_bytes(file, address, size):
    for segment in elf_segments(file):
        if segment[0] == 1 and segment[3] <= address and address + size <= segment[3] + segment[5]:
            file.seek(segment[2] + address - segment[3])
            return file.read(size)
    raise AssertionError("ELF identity bytes are absent from the actual core")


def inspect_core(core, wait, phase):
    assert phase["pid"] == wait["pid"] and phase["ppid"] == wait["launcherPid"], "PID/parent mismatch"
    assert wait["signal"] == 11 and wait["returncode"] == -11, "Wrong wait signal"
    assert core.name == f"core.{wait['pid']}" and wait["startedNs"] <= core.stat().st_mtime_ns <= wait["endedNs"], "Core PID/time mismatch"
    with core.open("rb") as file:
        notes = elf_notes(file)
        pid = [struct.unpack_from("<i", data, 24)[0] for owner, kind, data, *_ in notes if owner == b"CORE" and kind == 3]
        signals = [struct.unpack_from("<i", data)[0] for _, kind, data, *_ in notes if kind == 0x53494749]
        assert pid == [wait["pid"]] and signals == [11], "Core process/signal notes mismatch"
        entries = []
        for _, kind, data, *_ in notes:
            if kind != 0x46494C45:
                continue
            count, page = struct.unpack_from("<QQ", data)
            names = data[16 + 24 * count:].rstrip(b"\0").split(b"\0")
            assert len(names) == count
            for index, name in enumerate(names):
                start, end, offset = struct.unpack_from("<QQQ", data, 16 + 24 * index)
                entries.append((start, end, offset * page, name.decode()))
        threads = [data for owner, kind, data, *_ in notes if owner == b"CORE" and kind == 1]
        assert threads and struct.unpack_from("<h", threads[0], 12)[0] == 11
        fault_pc = struct.unpack_from("<Q", threads[0], 240)[0]
        fault_maps = [entry for entry in entries if entry[0] <= fault_pc < entry[1]]
        assert len(fault_maps) == 1 and any(image["filename"] == fault_maps[0][3] for image in phase["images"]), "Faulting mapped ELF identity unavailable"
        identities = []
        for image in phase["images"]:
            assert image["mappedIdentityEqual"], "Mapped backing device/inode changed"
            retained = Path(image["retained"])
            assert digest(retained.read_bytes()) == image["sha256"], "Retained image changed"
            with retained.open("rb") as backing:
                ids = [note for note in elf_notes(backing) if note[0] == b"GNU" and note[1] == 3]
            assert len(ids) == 1, "Mapped ELF build identity unavailable"
            _, _, identity, offset, raw_note = ids[0]
            mapped = [entry for entry in entries if entry[3] == image["filename"] and entry[2] <= offset and offset + len(raw_note) <= entry[2] + entry[1] - entry[0]]
            assert len(mapped) == 1, "Core/file mapping is ambiguous or absent"
            start, _, file_offset, _ = mapped[0]
            assert core_bytes(file, start + offset - file_offset, len(raw_note)) == raw_note, "Core mapped ELF identity mismatch"
            identities.append({"filename": image["filename"], "sha256": image["sha256"], "buildId": identity.hex()})
    return {"pid": wait["pid"], "signal": 11, "core": str(core),
            "sha256": file_digest(core), "faultPc": hex(fault_pc), "faultThread": struct.unpack_from("<i", threads[0], 32)[0], "mappedElfIdentities": identities}


def validate(directory, root):
    verify_source(root)
    cases = [json.loads(line) for line in (root / "tests/formal/impeto/fixtures/slot-reference.cases.jsonl").read_text().splitlines()]
    reference = [json.loads(line) for line in (root / "tests/formal/impeto/fixtures/slot-reference.behavior.jsonl").read_text().splitlines()]
    packets = sorted([(child, json.loads((child / "wait.json").read_text())) for child in (directory / "children").iterdir()], key=lambda pair: pair[1]["startedNs"])
    expected = [(case, behavior, backend) for case, behavior in zip(cases, reference) for backend in ["vdom", "vapor"]]
    assert len(cases) == len(reference) == 8 and 0 < len(packets) <= len(expected)
    failures = []
    for index, (child, wait) in enumerate(packets):
        assert not wait["relayErrors"]
        assert set(wait["raw"]) == {"stdin.bin", "stdout.bin", "stderr.bin"}, "Missing raw stream receipt"
        assert json.loads((child / "source.json").read_text()) == {"sha": SOURCE, "tree": TREE, "files": HASHES}
        for name, receipt in wait["raw"].items():
            data = (child / name).read_bytes()
            assert receipt == {"bytes": len(data), "sha256": digest(data)}, "Missing/changed raw stream"
        case, behavior, backend = expected[index]
        supplied = json.loads((child / "stdin.bin").read_bytes())
        assert supplied["backend"] == backend and supplied["identities"] is True and isinstance(supplied["code"], str)
        assert {key: supplied[key] for key in ["context", "steps", "slots"]} == case["scenario"]
        assert case["name"] == behavior["name"]
        phases = [json.loads(path.read_text()) for path in sorted(child.glob("identity-*.json"))]
        assert phases and all(phase["pid"] == wait["pid"] and phase["ppid"] == wait["launcherPid"] and phase["sourceSha"] == SOURCE for phase in phases), "Missing/mismatched PID image custody"
        assert len({phase["startTicks"] for phase in phases}) == 1, "PID was reused"
        phase = phases[-1]
        executable = phase["executable"]
        assert executable["procLink"] == "/proc/self/exe" and digest(Path(executable["retained"]).read_bytes()) == executable["sha256"]
        assert any(image["realpath"] == executable["realpath"] and image["sha256"] == executable["sha256"] for image in phase["images"]), "Running executable/mapping mismatch"
        try:
            reference_equal = json.loads((child / "stdout.bin").read_bytes()) == behavior["trace"]
        except (ValueError, UnicodeError):
            reference_equal = False
        if wait["signal"] == 11:
            core = directory / "cores" / f"core.{wait['pid']}"
            # Backtrace retention precedes strict core/image qualification.
            subprocess.run(["gdb", "--batch", "-iex", f"set sysroot {directory / 'sysroot'}", "-ex", "set pagination off", "-ex", "p $_siginfo", "-ex", "info files", "-ex", "info proc mappings", "-ex", "info sharedlibrary", "-ex", "thread apply all bt full", executable["retained"], str(core)], stdout=(child / "core.backtrace").open("wb"), stderr=subprocess.STDOUT, check=False)
            receipt = inspect_core(core, wait, phase)
            write_json(child / "core-identity.json", receipt)
        if wait["returncode"] != 0:
            failures.append({"index": index, "case": case["name"], "backend": backend, "pid": wait["pid"], "signal": wait["signal"], "wholeReferenceEqual": reference_equal, "originalRowReproduced": case["name"] == "named-prop" and backend == "vapor" and wait["signal"] == 11 and reference_equal})
            assert index == len(packets) - 1, "Original fatal assertion did not stop the test"
        else:
            assert reference_equal, "Original whole snapshot oracle failed"
    summary = {"source": SOURCE, "planned": 16, "observed": len(packets), "failures": failures,
               "historicalCause": None, "wholeCoverageAcceptance": False}
    write_json(directory / "summary.json", summary)
    print(json.dumps(summary))
    assert not failures and len(packets) == 16 and (directory / "cargo.status").read_text().strip() == "0", "Original negative remains fatal"


if __name__ == "__main__":
    if Path(sys.argv[0]).name == "node":
        launch()
    elif sys.argv[1] == "verify-source":
        print(json.dumps(verify_source(Path.cwd())))
    elif sys.argv[1] == "validate":
        validate(Path(sys.argv[2]), Path.cwd())
    else:
        raise AssertionError("Unsupported diagnostic operation")
