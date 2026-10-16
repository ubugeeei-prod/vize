"""Explicit diagnostic only: one original runner, one authenticated worker."""
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import signal
import subprocess
import sys
sys.dont_write_bytecode = True
import threading
import time

from owned_worker_identity import PACKAGE, file_identity, prepare_profile, providers, same_birth, select, state

FILES = ["owned_worker_capture.py", "owned_worker_identity.py", "owned_worker_gdb.py", "owned_worker_probe.mjs", "owned_worker_runner.py", "owned_worker_supervisor.py"]
SOURCE_CENSUS = ["native-fault-evidence.ts", "native-fault-core.ts", "native-fault-support.ts"]
LIVE_LAWS = "owned_worker_live_laws.py"
OLD_HEAD = "f2b31f4a8f1b677629c29c1be4f4ebfb68fc800e"
OLD_SOURCE = "52c78c8865237e8503a592f13029231b55da0537"
BEGIN, END = b"VIZE_OWNED_WORKER_BEGIN\n", b"VIZE_OWNED_WORKER_END\n"


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def git_identity():
    head, tree = subprocess.check_output(["git", "--no-replace-objects", "rev-parse", "HEAD", "HEAD^{tree}"], text=True).splitlines()
    diff = subprocess.check_output(["git", "--no-replace-objects", "diff", "--binary", "HEAD"])
    return {"head": head, "tree": tree, "diffSha256": hashlib.sha256(diff).hexdigest(),
            "trackedStatus": subprocess.check_output(["git", "--no-replace-objects", "status", "--porcelain=v1", "--untracked-files=no"], text=True).splitlines()}


def stage(source):
    if not re.fullmatch(r"[0-9a-f]{40}", source) or source in [OLD_HEAD, OLD_SOURCE]:
        raise ValueError("Only a fresh full source SHA is eligible")
    identity = "-".join(os.environ[key] for key in ["GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_JOB"])
    if not re.fullmatch(r"[0-9]+-[0-9]+-[a-zA-Z0-9_-]+", identity):
        raise ValueError("Unknown diagnostic run identity")
    output = Path(os.environ["RUNNER_TEMP"]) / ("owned-worker-fault-" + identity)
    output.mkdir(mode=0o700, exist_ok=False)
    (output / "public").mkdir()
    for filename in FILES:
        shutil.copyfile(Path(__file__).with_name(filename), output / filename)
    for filename in SOURCE_CENSUS:
        shutil.copyfile(Path.cwd() / ".github/actions/test-js-packages-with-history" / filename, output / filename)
    shutil.copyfile(Path.cwd() / "tests/tooling/fixtures" / LIVE_LAWS, output / LIVE_LAWS)
    receipt = {"orchestration": git_identity(), "physicalSource": source,
               "helpers": [file_identity(output / name) for name in FILES + SOURCE_CENSUS + [LIVE_LAWS]],
               "priorFailure": {"reportedHead": OLD_HEAD, "physicalSource": OLD_SOURCE,
                                "run": 38161052557, "attempt": 1, "job": 114534383424},
               "qualification": "diagnostic scheduling differs; no qualification or timing credit",
               "privacy": "private argv, environment, profile and GDB startup excluded"}
    write_json(output / "public/orchestration.json", receipt)
    with Path(os.environ["GITHUB_OUTPUT"]).open("a") as stream:
        stream.write("directory=" + str(output) + "\nsource-sha=" + source + "\n")


def authenticate(output):
    manifest = json.loads((output / "public/orchestration.json").read_text())
    actual = git_identity()
    if actual["head"] != manifest["physicalSource"] or actual["trackedStatus"]:
        raise ValueError("Diagnostic source SHA or clean checkout differs")
    for item in manifest["helpers"]:
        if file_identity(item["path"]) != item:
            raise ValueError("Reviewed orchestration helper changed")
    write_json(output / "public/source-authentication.json", actual)


class Worker:
    def __init__(self, identity, ops=signal, read_state=state, open_fd=os.pidfd_open if hasattr(os, "pidfd_open") else None, close_fd=os.close, read_tracer=None):
        self.identity, self.ops, self.read_state = identity, ops, read_state
        self.close_fd, self.fd, self.stopped = close_fd, None, False
        self.events = []
        self.read_tracer = read_tracer or self.tracer
        if open_fd is None:
            raise ValueError("pidfd API unavailable")
        self.check()
        self.fd = open_fd(identity["pid"])
        try:
            self.check()
            self.ops.pidfd_send_signal(self.fd, 0)
        except BaseException:
            self.close_fd(self.fd)
            self.fd = None
            raise

    def check(self):
        current = self.read_state(self.identity["pid"])
        if not same_birth(current, self.identity):
            raise ValueError("Owned worker birth changed")
        return current

    def tracer(self):
        lines = (Path("/proc") / str(self.identity["pid"]) / "status").read_text().splitlines()
        values = [line.split()[1] for line in lines if line.startswith("TracerPid:")]
        if len(values) != 1:
            raise ValueError("Actual worker tracer identity unavailable")
        return int(values[0])

    def stop(self):
        self.check()
        self.ops.pidfd_send_signal(self.fd, signal.SIGSTOP)
        self.stopped = True
        deadline = time.monotonic() + 1
        while time.monotonic() < deadline:
            if self.check()["state"] in ["T", "t"]:
                self.events.append("owned-worker-stopped")
                return
            time.sleep(.01)
        raise TimeoutError("Owned worker did not stop")

    def close(self):
        if self.fd is None:
            return
        try:
            if self.stopped:
                self.ops.pidfd_send_signal(self.fd, 0)
                self.check()
                if self.read_tracer() != 0:
                    raise ValueError("Owned worker still has an active tracer")
                self.check()
                self.ops.pidfd_send_signal(self.fd, signal.SIGCONT)
                self.events.append("owned-worker-continued")
        except ProcessLookupError:
            self.events.append("ESRCH-observation-gap")
        except (OSError, ValueError) as error:
            self.events.append("resume-refused-" + type(error).__name__)
        finally:
            self.close_fd(self.fd)
            self.fd = None


def read_markers(stream, packets, receipt, private=None):
    digest, count, active, body = hashlib.sha256(), 0, False, bytearray()
    private_digest, private_count = hashlib.sha256(), 0
    framing = {"begin": 0, "end": 0, "nonBoundaryBegin": 0}
    while True:
        line = stream.readline(131073)
        if not line:
            break
        digest.update(line)
        count += len(line)
        if private is not None:
            selected = line[:max(0, 131072 - private_count)]
            private.write(selected); private_digest.update(selected); private_count += len(selected)
        if line == BEGIN:
            framing["begin"] = min(64, framing["begin"] + 1)
            if active:
                receipt["markerGap"] = "NestedMarker"
            active, body = True, bytearray()
        elif line == END and active:
            framing["end"] = min(64, framing["end"] + 1)
            try:
                packet = json.loads(body)
                if not isinstance(packet, dict) or packet.get("status") not in ["attached", "SIGSEGV", "exited", "detached", "unavailable", "debugger-cleanup", "debugger-phase"]:
                    raise ValueError("Malformed diagnostic packet")
                if "argv" in packet or "environment" in packet:
                    raise ValueError("Private fields in diagnostic packet")
                packets.put_nowait(packet)
            except (ValueError, queue.Full):
                receipt["markerGap"] = "MalformedOrExcessPacket"
            active, body = False, bytearray()
        elif active:
            if len(body) + len(line) <= 131072:
                body.extend(line)
            else:
                receipt["markerGap"] = "DiagnosticTextExceedsBound"
                active = False
        elif BEGIN.strip() in line:
            framing["nonBoundaryBegin"] = min(64, framing["nonBoundaryBegin"] + 1)
            receipt["markerGap"] = "MarkerNotAtBoundary"
    if active:
        receipt["markerGap"] = "FinalPartialMarker"
    receipt["gdbStreamAuthority"] = {"bytes": count, "sha256": digest.hexdigest(),
                                     "selection": "bounded diagnostic markers only", "framing": framing}
    if private is not None:
        receipt["privateTranscript"] = {"retainedBytes": private_count, "sha256": private_digest.hexdigest(),
                                        "limit": 131072, "truncated": count > private_count, "uploaded": False}


def marker_reader(stream, packets, receipt, private=None):
    try:
        if private is None: read_markers(stream, packets, receipt)
        else:
            with os.fdopen(os.open(private, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as selected:
                read_markers(stream, packets, receipt, selected)
    except Exception as error:
        receipt["readerGap"] = type(error).__name__


def start_gdb(output, worker, leader, profile, root, receipt):
    private = output / "gdb-private"
    private.mkdir(mode=0o700)
    copied = ["owned_worker_gdb.py", "owned_worker_identity.py", "owned_worker_capture.py", "owned_worker_supervisor.py"]
    for name in copied:
        shutil.copyfile(output / name, private / name)
    receipt["gdbHelperIdentity"] = [file_identity(private / name) for name in copied]
    config = {"worker": worker, "leader": leader, "profile": profile, "root": str(root)}
    write_json(private / "attach-private.json", config)
    (private / "attach-private.json").chmod(0o600)
    child = subprocess.Popen(["sudo", "-n", "python3", str(private / "owned_worker_supervisor.py")],
                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    packets = queue.Queue(maxsize=16)
    reader = threading.Thread(target=marker_reader, args=(child.stdout, packets, receipt, private / "stdout-private.txt"), daemon=True)
    reader.start()
    return child, packets, reader


def close_gdb(child, reader, receipt):
    if child is None:
        return
    if child.poll() is None:
        child.send_signal(signal.SIGINT)  # Only our unreaped debugger child.
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                # The root supervisor owns bounded cleanup and parent-death
                # handling. Never kill the sudo monitor and orphan its debugger.
                receipt["gdbGap"] = "SupervisorCleanupNotObserved"
                try:
                    child.wait(timeout=100)
                except subprocess.TimeoutExpired:
                    receipt["gdbGap"] = "SupervisorUnreaped; diagnostic unavailable"
    reader.join(timeout=2)
    receipt["gdbReturnCode"] = child.returncode
    if reader.is_alive():
        receipt["gdbGap"] = "DebuggerReaderUnavailable"


def main(argv):
    output = Path(__file__).parent.resolve()
    if argv[0] == "stage":
        return stage(argv[1])
    if argv == ["authenticate"]:
        return authenticate(output)
    if argv == ["controls"]:
        if sys.platform != "linux" or os.environ.get("VIZE_OWNED_WORKER_CAPTURE") != "true":
            raise ValueError("Linux controls require explicit opt-in")
        from owned_worker_runner import controls_only
        return controls_only(output)
    if argv[:1] != ["--"] or len(argv) < 2:
        raise ValueError("Expected -- and the original runner command")
    command = argv[1:]
    if sys.platform != "linux" or os.environ.get("VIZE_OWNED_WORKER_CAPTURE") != "true":
        os.execvp(command[0], command)
    from owned_worker_runner import observe
    status = observe(command, output)
    if status < 0:
        if -status not in [signal.SIGKILL, signal.SIGSTOP]:
            signal.signal(-status, signal.SIG_DFL)
        os.kill(os.getpid(), -status)
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
