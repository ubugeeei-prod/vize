"""Observe one unchanged command; core capture requires a verified private guest."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time

HASH_MEASUREMENTS = {"bytes": 0, "seconds": 0.0}

if sys.platform == "linux":
    import resource


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def read_text(path):
    try:
        return Path(path).read_text().strip()
    except OSError:
        return None


def sha256_file(path):
    started = time.monotonic()
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
            HASH_MEASUREMENTS["bytes"] += len(block)
    HASH_MEASUREMENTS["seconds"] += time.monotonic() - started
    return digest.hexdigest()


def run_capture(argv, gated=False):
    result = {"argv": argv}
    with tempfile.TemporaryDirectory(prefix="native-fault-text-") as directory:
        paths = [Path(directory) / name for name in ["stdout", "stderr"]]
        with paths[0].open("wb") as stdout, paths[1].open("wb") as stderr:
            try:
                child = subprocess.run(argv, stdout=stdout, stderr=stderr, timeout=30)
                result["status"] = child.returncode
            except (OSError, subprocess.TimeoutExpired) as error:
                result["unavailable"] = type(error).__name__
        for name, path in zip(["stdout", "stderr"], paths):
            body, public_bytes = b"", 0
            with path.open("rb") as stream:
                admitted = not gated
                for line in stream:
                    if gated and line == b"VIZE_NATIVE_FAULT_BEGIN\n":
                        admitted = True
                        continue
                    if gated and line == b"VIZE_NATIVE_FAULT_END\n":
                        admitted = False
                        continue
                    if admitted and (not gated or name == "stdout"):
                        public_bytes += len(line)
                        body += line[:max(0, 262144 - len(body))]
            result[name] = body.decode("utf-8", errors="replace")
            result[name + "Authority"] = {"bytes": path.stat().st_size,
                "sha256": sha256_file(path), "publicBytes": public_bytes,
                "selection": "diagnostic markers; startup/arguments excluded" if gated else "whole stream",
                "truncated": public_bytes > len(body)}
    return result


def source_snapshot():
    head = run_capture(["git", "rev-parse", "HEAD", "HEAD^{tree}"])
    listing = subprocess.check_output(["git", "ls-files", "--stage", "-z"])
    files = []
    for entry in listing.split(b"\0"):
        if not entry:
            continue
        authority, filename = entry.decode().split("\t", 1)
        mode, blob, stage = authority.split()
        if stage != "0":
            raise ValueError("Source has unresolved index entries")
        path = Path(filename)
        item = {"file": filename, "mode": mode, "blob": blob}
        if mode == "160000":
            item["gitlink"] = blob
        else:
            try:
                if mode == "120000":
                    body = os.readlink(path).encode()
                    item.update(bytes=len(body), sha256=hashlib.sha256(body).hexdigest())
                else:
                    item.update(bytes=path.stat().st_size, sha256=sha256_file(path))
            except OSError as error:
                item["unavailable"] = type(error).__name__
        files.append(item)
    diff = subprocess.check_output(["git", "diff", "--binary", "HEAD"])
    return {"git": head, "files": files, "diffSha256": hashlib.sha256(diff).hexdigest()}


def private_guest():
    vm = run_capture(["systemd-detect-virt", "--vm"])
    container = run_capture(["systemd-detect-virt", "--container"])
    dmi = {name: read_text("/sys/class/dmi/id/" + name)
           for name in ["sys_vendor", "product_name", "bios_vendor"]}
    allowed = (vm.get("status") == 0 and vm.get("stdout", "").strip() == "kvm"
               and container.get("status") == 1
               and any("firecracker" in (value or "").lower() for value in dmi.values()))
    return {"privateFirecrackerGuest": allowed, "vm": vm, "container": container,
            "dmi": dmi, "bootId": read_text("/proc/sys/kernel/random/boot_id"),
            "cgroups": read_text("/proc/1/cgroup"), "kernel": os.uname().release,
            "cpuCount": os.cpu_count()}


def process_table():
    processes = {}
    for directory in Path("/proc").iterdir():
        if not directory.name.isdigit():
            continue
        try:
            stat = (directory / "stat").read_text()
            tail = stat[stat.rindex(")") + 2:].split()
            processes[int(directory.name)] = (int(tail[1]), tail[19])
        except (OSError, ValueError, IndexError):
            continue  # A vanished PID is an observation gap, never absence proof.
    return processes


def observe_descendants(leader, seen, retained_providers, output):
    processes = process_table()
    descendants = {leader}
    while True:
        added = {pid for pid, (parent, _) in processes.items() if parent in descendants}
        if added.issubset(descendants):
            break
        descendants.update(added)
    for pid in descendants:
        if pid not in processes:
            continue
        directory = Path("/proc") / str(pid)
        try:
            executable = os.readlink(directory / "exe")
            if Path(executable).name != "node" and pid != leader:
                continue
            maps = (directory / "maps").read_text()
            mapped = {fields[5]: int(fields[4]) for line in maps.splitlines()
                      for fields in [line.split(None, 5)] if len(fields) == 6
                      and "x" in fields[1] and fields[5].startswith("/")}
            identity = (pid, processes[pid][1], executable, tuple(sorted(mapped.items())))
            if identity in seen:
                continue
            providers = []
            for filename, mapped_inode in sorted(mapped.items()):
                item = {"path": filename, "mappedInode": mapped_inode}
                try:
                    path = Path(filename)
                    stat = path.stat()
                    if stat.st_ino != mapped_inode:
                        raise ValueError("Mapped provider inode changed")
                    key = (filename, stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns)
                    if key not in retained_providers:
                        sha = sha256_file(path)
                        after = path.stat()
                        if key[1:] != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
                            raise ValueError("Provider changed during hashing")
                        retained_providers[key] = dict(item, sha256=sha, bytes=stat.st_size)
                    item = retained_providers[key]
                except (OSError, ValueError) as error:
                    item["unavailable"] = type(error).__name__
                providers.append(item)
            packet = {"pid": pid, "startTicks": processes[pid][1], "executable": executable,
                      "cwd": os.readlink(directory / "cwd"), "maps": maps, "providers": providers}
            current = process_table().get(pid)
            if current is None or current[1] != processes[pid][1] or os.readlink(directory / "exe") != executable:
                continue
            write_json(output / "processes" / (str(pid) + "-" + str(len(seen)) + ".json"), packet)
            seen.add(identity)
        except OSError:
            continue


def diagnose_core(core, receipt, output):
    details = {"file": core.name, "bytes": core.stat().st_size, "sha256": sha256_file(core),
               "rawCore": "runner-local; not published"}
    from native_fault_core import producer
    try:
        identity = producer(core, output, sha256_file)
    except (OSError, ValueError, KeyError) as error:
        details["producerUnavailable"] = type(error).__name__
        receipt["cores"].append(details)
        return
    details["producer"] = identity
    if shutil.which("gdb") is None:
        receipt["gdbInstall"] = run_capture(["sudo", "-n", "apt-get", "install", "-y", "--no-install-recommends", "gdb"])
    if shutil.which("gdb"):
        result = run_capture(["gdb", "--batch", "-nx", "-iex", "set auto-load off",
                              "-iex", "set debuginfod enabled off", "-ex", "set pagination off",
                              "-ex", "set print frame-arguments none", "-ex", "set print entry-values no",
                              "-ex", "echo VIZE_NATIVE_FAULT_BEGIN\\n", "-ex", "info target",
                              "-ex", "thread apply all bt", "-ex", "info sharedlibrary",
                              "-ex", "info registers", "-ex", "echo VIZE_NATIVE_FAULT_END\\n",
                              identity["path"], str(core)], gated=True)
        write_json(output / (core.name + "-gdb.json"), result)
        details["gdbStatus"] = result.get("status")
    else:
        details["gdbUnavailable"] = True
    receipt["cores"].append(details)


def attempt(receipt, name, action):
    started = time.monotonic()
    try:
        return action()
    except Exception as error:
        receipt.setdefault("observationGaps", {})[name] = type(error).__name__
        return None
    finally:
        times = receipt.setdefault("observationWalltimeSeconds", {})
        times[name] = times.get(name, 0.0) + time.monotonic() - started


def prepare_observation(command):
    identity = "-".join(os.environ[key] for key in ["GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_JOB"])
    output = Path(os.environ["RUNNER_TEMP"]) / ("js-native-fault-evidence-" + identity)
    output.mkdir(parents=True, exist_ok=False)
    helper = Path(__file__).resolve()
    receipt = {"command": command, "cwd": str(Path.cwd()), "startedAt": time.time(),
               "orchestrationSha": os.environ.get("GITHUB_SHA"),
               "helpers": [{"path": str(path), "sha256": sha256_file(path)}
                           for path in [helper, helper.with_name("native_fault_core.py")]],
               "sampling": "read-only /proc descendants; sampled maps are not complete-call evidence",
               "coreCapture": "unavailable", "cores": []}
    receipt["controlledEnvironment"] = {name: os.environ.get(name) for name in [
        "NODE_OPTIONS", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS", "VIZE_TEST_REQUIRE_TSGO", "VIZE_TEST_DISABLE_TSGO",
        "VIZE_NUXT_CONFIG_ITERATIONS", "VIZE_VUE_RUNTIME_PRODUCTION", "VIZE_VUE_RUNTIME_DEVTOOLS"]}
    return output, receipt


def observe_command(command, output, receipt):
    attempt(receipt, "source-before", lambda: write_json(output / "source-before.json", source_snapshot()))
    guest = attempt(receipt, "guest", private_guest) or {"privateFirecrackerGuest": False}
    attempt(receipt, "guest-packet", lambda: write_json(output / "guest.json", guest))
    original_pattern = read_text("/proc/sys/kernel/core_pattern")
    core_directory = Path(os.environ["RUNNER_TEMP"]) / output.name.replace("js-native-fault-evidence-", "js-native-fault-cores-", 1)
    pattern_changed = False
    if guest["privateFirecrackerGuest"] and original_pattern is not None:
        try:
            soft, hard = resource.getrlimit(resource.RLIMIT_CORE)
            if hard == 0:
                raise ValueError("Hard core limit is zero")
            core_directory.mkdir(exist_ok=False)
            pattern_changed = True  # Restore even when the setup observation itself fails.
            receipt["coreSetup"] = run_capture(["sudo", "-n", "sysctl", "-w", "kernel.core_pattern=" + str(core_directory / "core.%p")])
            if receipt["coreSetup"].get("status") == 0:
                pattern_changed = True
                resource.setrlimit(resource.RLIMIT_CORE, (hard, hard))
                receipt.update(coreCapture="enabled-private-guest", originalCoreLimit=[soft, hard])
        except (OSError, ValueError) as error:
            receipt["coreLimit"] = {"unavailable": type(error).__name__}
    seen, providers = set(), {}
    child = None
    try:
        receipt["commandStartedAt"] = time.time()
        child = subprocess.Popen(command)  # Original argv/env/stdin/stdout/stderr, once.
        receipt["pid"] = child.pid
        previous_handlers = {}
        def forward(signum, frame):
            receipt["forwardedSignal"] = signum
            if child.poll() is None:
                child.send_signal(signum)
        for signum in [signal.SIGINT, signal.SIGTERM]:
            previous_handlers[signum] = signal.signal(signum, forward)
        while child.poll() is None:
            attempt(receipt, "process-maps", lambda: observe_descendants(child.pid, seen, providers, output))
            time.sleep(0.25)
        receipt.update(commandReturnCode=child.returncode, commandFinishedAt=time.time())
    finally:
        for signum, handler in locals().get("previous_handlers", {}).items():
            signal.signal(signum, handler)
        receipt.update(finishedAt=time.time(), sampledMapPackets=len(seen))
        if pattern_changed:
            attempt(receipt, "restore", lambda: receipt.update(coreRestore=run_capture(["sudo", "-n", "sysctl", "-w", "kernel.core_pattern=" + original_pattern])))
        attempt(receipt, "outcome", lambda: write_json(output / "outcome.json", receipt))
    cores = attempt(receipt, "core-inventory", lambda: sorted(core_directory.glob("core.*"))) or []
    for core in cores:
        attempt(receipt, "core-" + core.name, lambda: diagnose_core(core, receipt, output))
    attempt(receipt, "source-after", lambda: write_json(output / "source-after.json", source_snapshot()))
    receipt["observerFileHash"] = dict(HASH_MEASUREMENTS)
    receipt["observerFinishedAt"] = time.time()
    attempt(receipt, "outcome-final", lambda: write_json(output / "outcome.json", receipt))
    return child.returncode


def main(argv):
    if not argv or argv[0] != "--" or len(argv) < 2:
        raise ValueError("Expected -- followed by the original command")
    command = argv[1:]
    if sys.platform != "linux" or os.environ.get("GITHUB_ACTIONS") != "true":
        os.execvp(command[0], command)
    # Setup refusal must still execute the original command, with unchanged status.
    try:
        output, receipt = prepare_observation(command)
    except Exception:
        os.execvp(command[0], command)
    status = observe_command(command, output, receipt)
    if status < 0:
        attempt(receipt, "observer-core-limit", lambda: resource.setrlimit(resource.RLIMIT_CORE, (0, resource.getrlimit(resource.RLIMIT_CORE)[1])))
        if -status not in [signal.SIGKILL, signal.SIGSTOP]:
            signal.signal(-status, signal.SIG_DFL)
        os.kill(os.getpid(), -status)
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
