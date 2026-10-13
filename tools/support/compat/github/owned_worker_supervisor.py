"""Root supervisor owns only its debugger child and the verified worker pidfd."""
import ctypes
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import signal
import subprocess
import sys
sys.dont_write_bytecode = True
import threading
import time

from owned_worker_capture import Worker
from owned_worker_identity import HASH_METRICS, file_identity, same_birth, select, state


def parent_death():
    parent = os.getppid()
    if ctypes.CDLL(None, use_errno=True).prctl(1, signal.SIGTERM, 0, 0, 0) != 0:
        raise OSError("Parent-death signal setup failed")
    if os.getppid() != parent:
        raise ProcessLookupError("Parent vanished during setup")


def publish(value):
    print("VIZE_OWNED_WORKER_BEGIN", flush=True)
    print(json.dumps(value, separators=(",", ":")), flush=True)
    print("VIZE_OWNED_WORKER_END", flush=True)


def input_reader(stream, packets):
    while True:
        body = stream.readline(32)
        if body not in ["captured\n", "detach\n"]:
            packets.put("detach")
            return
        packets.put(body.strip())


def debugger_errors(stream, directory, receipt):
    digest, count, frames, errors = hashlib.sha256(), 0, [], []
    allowed = {name: file_identity(directory / name)["sha256"] for name in
               ["owned_worker_gdb.py", "owned_worker_identity.py", "owned_worker_capture.py", "owned_worker_supervisor.py"]}
    kinds = {"NameError", "ModuleNotFoundError", "ImportError", "ValueError", "RuntimeError",
             "AttributeError", "TypeError", "OSError", "PermissionError", "ProcessLookupError", "SyntaxError", "gdb.error"}
    while True:
        line = stream.readline(131073)
        if not line: break
        digest.update(line); count += len(line)
        text = line.decode("utf8", errors="replace")
        frame = re.fullmatch(r'\s*File "([^"\n]+)", line ([0-9]{1,6})(?:, in [^\n]*)?\n?', text)
        if frame and len(frames) < 16:
            path = Path(frame[1])
            if path.parent == directory and path.name in allowed:
                frames.append({"helper": path.name, "line": int(frame[2]), "sha256": allowed[path.name]})
        kind = text.split(":", 1)[0]
        if kind in kinds and len(errors) < 16: errors.append(kind)
    receipt["debuggerStderr"] = {"bytes": count, "sha256": digest.hexdigest(),
                                 "recognizedHelperFrames": frames, "recognizedErrorTypes": errors,
                                 "selection": "closed helper identity/type projection; other text excluded"}


def debugger_error_reader(stream, directory, receipt):
    try: debugger_errors(stream, directory, receipt)
    except Exception as error: receipt["gap"] = "DebuggerErrorReader" + type(error).__name__


def reap_debugger(child, receipt):
    if child.poll() is None:
        for name, method in [("SIGINT", lambda: child.send_signal(signal.SIGINT)), ("SIGTERM", child.terminate), ("SIGKILL", child.kill)]:
            receipt.setdefault("cleanupSignalAttempts", []).append({"signal": name, "atMonotonicNs": time.monotonic_ns()})
            method()  # Unreaped actual debugger child; no numeric PID lookup.
            try:
                child.wait(timeout=3)
                receipt["cleanupSignalAttempts"][-1]["wait"] = "reaped"
                break
            except subprocess.TimeoutExpired:
                receipt["cleanupSignalAttempts"][-1]["wait"] = "timeout"
                receipt["forcedDebuggerExit"] = True
    receipt["debuggerReturnCode"] = child.wait(timeout=0)
    receipt["debuggerReaped"] = True


def supervise(config, directory):
    child, worker, error_reader = None, None, None
    receipt = {"status": "debugger-cleanup", "debuggerReaped": False,
               "workerResumed": False, "numericAttachAtomic": False}
    cancelled = threading.Event()
    handlers = {}
    try:
        actor = config["worker"]["actor"]
        if os.geteuid() != 0 or int(os.environ["SUDO_UID"]) != actor["uid"]:
            raise ValueError("Unexpected privileged observer owner")
        for signum in [signal.SIGINT, signal.SIGTERM]:
            handlers[signum] = signal.signal(signum, lambda number, frame: cancelled.set())
        parent_death()
        for path in [directory, directory / "attach-private.json"]:
            if path.is_symlink() or path.stat().st_uid != actor["uid"] or path.stat().st_mode & 0o077:
                raise ValueError("Private debugger custody differs")
        if not same_birth(state(actor["pid"]), actor):
            raise ValueError("Worker birth changed before supervisor")
        selected = select(actor["pid"], config["leader"], config["profile"], Path(config["root"]), owner_uid=actor["uid"])
        if selected is None:
            raise ValueError("Private worker actor differs")
        worker = Worker(actor)
        worker.stopped = True  # Main helper has already proved this actor stopped.
        if worker.check()["state"] not in ["T", "t"]:
            raise ValueError("Owned worker was not stopped")
        packets = queue.Queue(maxsize=16)
        if cancelled.is_set():
            raise InterruptedError("Supervisor parent vanished")
        publish({"status": "debugger-phase", "phase": "supervisor-qualified"})
        child = subprocess.Popen(["gdb", "--batch", "-nx", "--nh", "-iex", "set auto-load off",
                                  "-iex", "set debuginfod enabled off", "-x", str(directory / "owned_worker_gdb.py")],
                                 stdin=subprocess.PIPE, stderr=subprocess.PIPE,
                                 preexec_fn=parent_death)
        error_reader = threading.Thread(target=debugger_error_reader, args=(child.stderr, directory, receipt), daemon=True)
        error_reader.start()
        threading.Thread(target=input_reader, args=(sys.stdin, packets), daemon=True).start()
        deadline = time.monotonic() + 90
        while child.poll() is None and not cancelled.is_set() and time.monotonic() < deadline:
            try:
                packet = packets.get(timeout=.1)
                if packet == "detach":
                    break
                child.stdin.write(b"captured\n")
                child.stdin.flush()
            except queue.Empty:
                pass
    except Exception as error:
        receipt["gap"] = type(error).__name__
    finally:
        if child is not None:
            try:
                (directory / "detach-requested").write_text("", encoding="ascii")
                reap_debugger(child, receipt)
            except Exception as error:
                receipt["gap"] = "Reap" + type(error).__name__
        if error_reader is not None:
            error_reader.join(timeout=2)
            if error_reader.is_alive(): receipt["gap"] = "DebuggerErrorReaderUnavailable"
        if worker is not None:
            worker.close()
            receipt["workerCleanup"] = worker.events
            receipt["workerResumed"] = "owned-worker-continued" in worker.events
        for signum, handler in handlers.items():
            signal.signal(signum, handler)
        receipt["hashing"] = dict(HASH_METRICS)
        publish(receipt)
    return 0 if receipt["debuggerReaped"] else 1


if __name__ == "__main__":
    directory = Path(__file__).parent.resolve()
    sys.exit(supervise(json.loads((directory / "attach-private.json").read_text()), directory))
