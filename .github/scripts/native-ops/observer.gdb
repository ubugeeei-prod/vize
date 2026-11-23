python
"""Event-stop diagnostics only; no deliberate inferior data/register edits.
GDB may install internal/software breakpoints. Results are never qualification.
"""
import hashlib
import json
import os
import re
import signal
from pathlib import Path
import time

import gdb

# The driver opens this file before starting GDB. Using that descriptor keeps
# the inferior's original working directory, arguments, and environment intact.
capture = Path(os.readlink("/proc/self/fd/1"))
if capture.name != "stdout.raw" or not capture.is_absolute():
    raise RuntimeError("expected the debugger's private stdout capture")
OUT = capture.parent
OUT.mkdir(parents=True, exist_ok=True)
EVENTS = OUT / "events.jsonl"
ENV_KEYS = (
    "PATH", "LD_PRELOAD", "RUST_MIN_STACK", "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS", "NODE_OPTIONS", "CARGO_BUILD_JOBS",
    "CARGO_TARGET_DIR", "RUSTUP_TOOLCHAIN", "RUSTUP_HOME", "CARGO_HOME",
    "RUSTUP_FORCE_ARG0", "RAYON_NUM_THREADS", "TOKIO_WORKER_THREADS",
    "VP_RUN_CONCURRENCY_LIMIT", "VIZE_NATIVE_OVERRIDE", "VIZE_NATIVE_PATH",
    "VIZE_PLUGIN_HOST_BUILD_ID", "GITHUB_SHA", "GITHUB_REF",
    "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT",
)
SECRET_KEYS = (
    "GITHUB_TOKEN", "GH_TOKEN", "NPM_TOKEN", "NODE_AUTH_TOKEN",
    "ACTIONS_RUNTIME_TOKEN", "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
)
SECRETS = [os.environ[key] for key in SECRET_KEYS if os.environ.get(key)]
CACHE = {}
PIDS = {}
GAPS = []
ROOT_NUM = gdb.selected_inferior().num
ROOT_EXIT = None
STARTED = time.monotonic_ns()


def public(value):
    if isinstance(value, str):
        for secret in SECRETS:
            value = value.replace(secret, "<redacted>")
        return value
    if isinstance(value, dict):
        return {key: public(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [public(item) for item in value]
    return value


def emit(kind, **fields):
    row = public({"event": kind, "monotonicNs": time.monotonic_ns(), **fields})
    with EVENTS.open("a", encoding="utf8") as stream:
        stream.write(json.dumps(row, ensure_ascii=True) + "\n")


def gap(operation, error):
    item = {"operation": operation, "error": type(error).__name__}
    GAPS.append(item)
    emit("observer-gap", **item)


def read(filename):
    return Path(filename).read_bytes()


def image(filename):
    with open(filename, "rb") as stream:
        st = os.fstat(stream.fileno())
        key = (st.st_dev, st.st_ino, st.st_size, st.st_mtime_ns, st.st_ctime_ns)
        if key not in CACHE:
            digest = hashlib.sha256()
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
            CACHE[key] = digest.hexdigest()
    return {
        "sha256": CACHE[key], "bytes": st.st_size, "device": st.st_dev,
        "inode": st.st_ino, "mtimeNs": st.st_mtime_ns, "ctimeNs": st.st_ctime_ns,
    }


def argv_packet(raw):
    values = [part.decode("utf8", "replace") for part in raw.split(b"\0") if part]
    projected = []
    hide_next = False
    for value in values:
        if hide_next:
            projected.append("<redacted>")
            hide_next = False
            continue
        if value in ("-e", "-p", "--eval", "--print", "--config"):
            projected.append(value)
            hide_next = True
            continue
        if re.match(r"^--?(token|password|secret|authorization|auth|header|otp)(=|$)", value, re.I) or value == "-H":
            flag, sep, _ = value.partition("=")
            projected.append(flag + "=<redacted>" if sep else flag)
            hide_next = not sep
            continue
        if re.search(r"authorization\s*[:=]|_authToken\s*=|password\s*=|token\s*=", value, re.I):
            projected.append("<sensitive-argument>")
            continue
        if re.match(r"^[^=]*(TOKEN|PASSWORD|SECRET|AUTH)[^=]*=", value, re.I):
            projected.append(value.split("=", 1)[0] + "=<redacted>")
            continue
        value = re.sub(r"(https?://)[^/@\s]+@", r"\1<redacted>@", value)
        if len(value) > 4096:
            value = "<long-argument sha256=" + hashlib.sha256(value.encode()).hexdigest() + ">"
        projected.append(value)
    return projected


def logical_tool(argv, environment, cwd):
    if not argv:
        return None
    requested = argv[0]
    name = Path(requested).name
    if name not in ("rustc", "rustup", "pnpm", "vp", "cargo", "node"):
        return None
    choices = [requested] if "/" in requested else [
        str(Path(part or ".") / requested)
        for part in environment.get("PATH", "").split(os.pathsep)
    ]
    for choice in choices:
        selected = Path(choice)
        if not selected.is_absolute():
            selected = Path(cwd) / selected
        if not os.access(selected, os.X_OK):
            continue
        chain = []
        current = selected
        for _ in range(32):
            if not current.is_symlink():
                break
            target = os.readlink(current)
            chain.append({"link": str(current), "target": target})
            current = Path(target) if os.path.isabs(target) else current.parent / target
        resolved = selected.resolve(strict=True)
        return {"requested": requested, "selected": str(selected),
                "symlinks": chain, "realpath": str(resolved), **image(resolved)}
    return {"requested": requested, "unresolved": True}


def snapshot(inferior, maps=False):
    pid = inferior.pid
    if not pid:
        return {"inferior": inferior.num, "pid": 0}
    PIDS[inferior.num] = pid
    proc = Path("/proc") / str(pid)
    raw_argv = read(proc / "cmdline")
    environment = {}
    for entry in read(proc / "environ").split(b"\0"):
        key, sep, value = entry.partition(b"=")
        name = key.decode("utf8", "replace")
        if sep and name in ENV_KEYS:
            environment[name] = value.decode("utf8", "replace")
    exe = proc / "exe"
    cwd = os.readlink(proc / "cwd")
    actual_argv = [part.decode("utf8", "replace") for part in raw_argv.split(b"\0") if part]
    result = {
        "inferior": inferior.num, "pid": pid,
        "executable": {"link": os.readlink(exe), **image(exe)},
        "cwd": cwd,
        "logicalToolAtExec": logical_tool(actual_argv, environment, cwd),
        "argvSHA256": hashlib.sha256(raw_argv).hexdigest(),
        "argv": argv_packet(raw_argv),
        "environment": {key: environment.get(key) for key in ENV_KEYS},
        "limits": read(proc / "limits").decode(),
        "status": "\n".join(
            line for line in read(proc / "status").decode().splitlines()
            if line.split(":", 1)[0] in ("Name", "Pid", "Tgid", "PPid")
        ),
        "personality": read(proc / "personality").decode().strip(),
    }
    if maps:
        result["maps"] = read(proc / "maps").decode()
    return result


def thread_packet(thread):
    return {
        "inferior": thread.inferior.num, "pid": thread.inferior.pid,
        "ptid": list(thread.ptid), "name": thread.name,
        "globalNumber": thread.global_num,
    }


def frame_packets():
    frames = []
    try:
        frame = gdb.newest_frame()
        for _ in range(64):
            if frame is None:
                break
            row = {"pc": hex(frame.pc()), "function": frame.name()}
            try:
                location = frame.find_sal()
                row["file"] = location.symtab.fullname() if location.symtab else None
                row["line"] = location.line
            except gdb.error:
                pass
            frames.append(row)
            frame = frame.older()
    except gdb.error as error:
        frames.append({"unwindUnavailable": type(error).__name__})
    return frames


def providers(pid, maps):
    result = []
    seen = set()
    for line in maps.splitlines():
        fields = line.split(None, 5)
        if len(fields) != 6 or "x" not in fields[1] or not fields[5].startswith("/"):
            continue
        address, permissions, offset, device, inode, name = fields
        key = (device, inode, name)
        if key in seen:
            continue
        seen.add(key)
        row = {"path": name, "mappedDevice": device, "mappedInode": int(inode)}
        mapped_file = Path("/proc") / str(pid) / "map_files" / address
        try:
            row["actualMappedImage"] = image(mapped_file)
        except (OSError, ValueError) as error:
            row["actualMappedImageUnavailable"] = type(error).__name__
            # Never relabel an on-disk replacement as the mapped image.
            try:
                current = image(name)
                row["currentDiskImage"] = current
                major, minor = (int(part, 16) for part in device.split(":"))
                row["currentInodeAndDeviceMatchMap"] = (
                    current["inode"] == int(inode)
                    and os.major(current["device"]) == major
                    and os.minor(current["device"]) == minor
                )
            except OSError as other:
                row["currentDiskImageUnavailable"] = type(other).__name__
        result.append(row)
    return result


def on_thread(event):
    try:
        thread = event.inferior_thread
        PIDS[thread.inferior.num] = thread.inferior.pid
        emit(type(event).__name__, thread=thread_packet(thread))
    except Exception as error:
        gap("thread-event", error)


def on_stop(event):
    try:
        inferior = gdb.selected_inferior()
        thread = gdb.selected_thread()
        sig = getattr(event, "stop_signal", None)
        packet = snapshot(inferior, maps=sig is not None)
        emit("stop", stopType=type(event).__name__, signal=sig,
             thread=thread_packet(thread) if thread else None, process=packet)
        if sig in ("SIGSEGV", "SIGABRT", "SIGBUS", "SIGILL"):
            emit("fault-state", signal=sig, pid=inferior.pid,
                 pc=hex(int(gdb.parse_and_eval("$pc"))),
                 thread=thread_packet(thread), frames=frame_packets(),
                 providers=providers(inferior.pid, packet.get("maps", "")),
                 registers=gdb.execute("info registers", to_string=True),
                 threads=gdb.execute("info threads", to_string=True))
    except Exception as error:
        gap("stop-event", error)


def on_exit(event):
    global ROOT_EXIT
    try:
        inferior = event.inferior
        pid = PIDS.get(inferior.num, inferior.pid)
        code = getattr(event, "exit_code", None)
        number = None
        if code is None:
            # GDB sets this from the actual terminal wait status before ExitedEvent.
            # Never infer a terminal signal from an earlier handled signal stop.
            if gdb.selected_inferior().num != inferior.num:
                raise RuntimeError("Exit event is not the selected inferior")
            value = gdb.convenience_variable("_exitsignal")
            if value is None or value.type.code == gdb.TYPE_CODE_VOID:
                raise RuntimeError("No authoritative terminal signal")
            number = int(value)
            if not 0 < number < 128:
                raise RuntimeError("Invalid terminal signal")
        packet = {"inferior": inferior.num, "pid": pid, "exitCode": code,
                  "signalNumber": number,
                  "signal": signal.Signals(number).name if number else None,
                  "terminalAuthority": "ExitedEvent.exit_code" if code is not None else "$_exitsignal"}
        emit("exit", **packet)
        if inferior.num == ROOT_NUM:
            ROOT_EXIT = packet
    except Exception as error:
        gap("exit-event", error)


settings = (
    "set confirm off", "set pagination off", "set startup-with-shell off",
    "set disable-randomization off", "set detach-on-fork off",
    "set follow-fork-mode parent", "set follow-exec-mode same",
    "set schedule-multiple on", "set non-stop off",
    "set auto-load python-scripts off", "set auto-load local-gdbinit off",
    "set auto-load gdb-scripts off", "set auto-solib-add off",
    "set print frame-arguments none", "set print entry-values no",
    "handle SIGSEGV stop print pass", "handle SIGABRT stop print pass",
    "handle SIGBUS stop print pass", "handle SIGILL stop print pass",
)
for command in settings:
    gdb.execute(command, to_string=True)
gdb.execute("set logging file " + str(OUT / "gdb.txt"), to_string=True)
gdb.execute("set logging overwrite on", to_string=True)
gdb.execute("set logging redirect on", to_string=True)
gdb.execute("set logging enabled on", to_string=True)
for command in ("catch fork", "catch vfork", "catch exec"):
    gdb.execute(command, to_string=True)
gdb.events.stop.connect(on_stop)
gdb.events.new_thread.connect(on_thread)
gdb.events.thread_exited.connect(on_thread)
gdb.events.exited.connect(on_exit)
emit("observer", qualified=False, gdbVersion=gdb.VERSION, settings=list(settings),
     footprint="ptrace stops and possible internal/software breakpoints; no deliberate inferior data/register edits")
try:
    gdb.execute("run")
    while True:
        # A descendant can fault after its original parent has already exited.
        live = [inferior for inferior in gdb.inferiors() if inferior.pid]
        if not live:
            break
        if not gdb.selected_inferior().pid:
            gdb.execute("inferior " + str(live[0].num), to_string=True)
        gdb.execute("continue")
except Exception as error:
    gap("execution-loop", error)
finally:
    remaining = [inferior.num for inferior in gdb.inferiors() if inferior.pid]
    if remaining:
        gap("incomplete-live-inferiors", RuntimeError("Execution loop ended early"))
        try:
            gdb.execute("detach inferiors " + " ".join(map(str, remaining)), to_string=True)
            emit("detach-after-observer-failure", inferiors=remaining)
        except Exception as error:
            gap("detach-after-observer-failure", error)
    summary = {
        "qualified": False, "rootExit": ROOT_EXIT, "observerGaps": GAPS,
        "remainingInferiors": remaining,
        "walltimeNs": time.monotonic_ns() - STARTED,
        "meaning": "forensic result only; no performance or product acceptance",
    }
    (OUT / "summary.json").write_text(json.dumps(public(summary), indent=2) + "\n")
end
