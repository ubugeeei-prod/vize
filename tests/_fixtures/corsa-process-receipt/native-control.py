#!/usr/bin/env python3
"""Test-only LSP transport; injected stderr belongs to the wrapper, not native."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import threading
import time

EOF_LINE = b"error handling method 'exit': EOF\n"
PREFIX = {
    "unexpected": b"unexpected native stderr\n",
    "nonutf8": b"wrapper\0\xffcontrol\n",
    "nonzero": b"",
    "sigsegv": b"",
    "timeout": b"",
}



def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON member")
        result[key] = value
    return result


def frames(buffer):
    while True:
        end = buffer.find(b"\r\n\r\n")
        if end < 0:
            return
        lengths = []
        for line in buffer[:end].split(b"\r\n"):
            name, separator, value = line.partition(b":")
            if name.lower() == b"content-length" and separator:
                lengths.append(int(value.strip()))
        if len(lengths) != 1 or not 0 <= lengths[0] <= 64 * 1024 * 1024:
            raise ValueError("invalid Content-Length")
        size = end + 4 + lengths[0]
        if len(buffer) < size:
            return
        frame = bytes(buffer[:size])
        del buffer[:size]
        try:
            message = json.loads(frame[end + 4:], object_pairs_hook=unique_object)
        except (ValueError, UnicodeError):
            message = None  # Invalid JSON is forwarded unchanged, never Exit-translated.
        yield frame, message


class Control:
    def __init__(self, directory):
        self.lock = threading.RLock()
        self.directory = directory
        self.files = {
            name: (directory / name).open("xb", buffering=0)
            for name in (
                "wrapper.stdin.bin", "native.stdin.bin", "native.stdout.bin",
                "native.stderr.bin", "wrapper.stdout.bin", "wrapper.stderr.bin",
                "wrapper.injected-stderr.bin", "translated-exit.frames.bin",
            )
        }
        self.errors = []
        self.events = []
        self.shutdown_id = None
        self.shutdown_ack = threading.Event()
        self.translated_exit = False
        self.stdin_complete = False
        self.stdout_complete = False
        self.stderr_complete = False

    def event(self, kind, **fields):
        with self.lock:
            self.events.append({"kind": kind, "monotonicNs": time.monotonic_ns(), **fields})

    def error(self, message):
        with self.lock:
            self.errors.append(str(message))

    def emit(self, stream, data, injected=False):
        with self.lock:
            self.files[f"wrapper.{stream}.bin"].write(data)
            if injected:
                self.files["wrapper.injected-stderr.bin"].write(data)
            try:
                view = memoryview(data)
                while view:
                    view = view[os.write(1 if stream == "stdout" else 2, view):]
            except OSError as error:
                self.error(f"wrapper {stream} forwarding: {error}")

    def drain(self, stream, pipe):
        buffer = bytearray()
        parse_failed = False
        try:
            while data := os.read(pipe.fileno(), 65536):
                self.files[f"native.{stream}.bin"].write(data)
                self.emit(stream, data)
                if stream == "stdout" and not parse_failed:
                    buffer.extend(data)
                    try:
                        for _, message in frames(buffer):
                            if isinstance(message, dict):
                                with self.lock:
                                    acknowledged = (
                                        self.shutdown_id is not None
                                        and message.get("id") == self.shutdown_id
                                        and message.get("jsonrpc") == "2.0"
                                        and "result" in message
                                        and message["result"] is None
                                        and "error" not in message
                                    )
                                if acknowledged:
                                    self.event("native-shutdown-ack", message=message)
                                    self.shutdown_ack.set()
                    except ValueError as error:
                        self.error(f"native stdout frame: {error}")
                        parse_failed = True  # Keep draining and forwarding every raw byte.
            if stream == "stdout" and buffer and not parse_failed:
                self.error("incomplete native stdout LSP frame")
            setattr(self, f"{stream}_complete", True)
        except Exception as error:
            self.error(f"native {stream} reader: {error}")
        finally:
            pipe.close()

    def forward(self, child, frame, message):
        if message == {"jsonrpc": "2.0", "method": "exit"}:
            self.files["translated-exit.frames.bin"].write(frame)
            if not self.shutdown_ack.is_set() or self.translated_exit:
                raise ValueError("Exit without exactly one acknowledged shutdown")
            self.event("wrapper-exit-to-native-stdin-eof", frameHex=frame.hex())
            self.translated_exit = True
            child.stdin.close()
            return
        if self.translated_exit:
            raise ValueError("frame after translated Exit")
        if isinstance(message, dict) and (
            set(message) == {"jsonrpc", "id", "method"}
            and message["jsonrpc"] == "2.0" and message["method"] == "shutdown"
        ):
            with self.lock:
                if self.shutdown_id is not None:
                    raise ValueError("duplicate shutdown request")
                self.shutdown_id = message["id"]
            self.event("forwarded-shutdown", message=message)
        view = memoryview(frame)
        while view:
            count = os.write(child.stdin.fileno(), view)
            self.files["native.stdin.bin"].write(view[:count])
            view = view[count:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--mode", choices=PREFIX, required=True)
    parser.add_argument("--control-dir", type=Path, required=True)
    parser.add_argument("--timeout-seconds", type=float, default=30)
    parser.add_argument("native_args", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    native = args.native.resolve(strict=True)
    native_args = args.native_args[1:] if args.native_args[:1] == ["--"] else args.native_args
    native_hash = hashlib.sha256(native.read_bytes()).hexdigest()
    if "--lsp" not in native_args:
        os.execv(str(native), [str(native), *native_args])
    if not 0 < args.timeout_seconds <= 60:
        raise ValueError("timeout must be greater than zero and at most 60 seconds")
    args.control_dir.mkdir(parents=True, exist_ok=True)
    control = Control(args.control_dir)
    started = time.monotonic_ns()
    control.emit("stderr", PREFIX[args.mode], injected=True)
    child = subprocess.Popen(
        [str(native), *native_args], stdin=subprocess.PIPE,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0,
    )
    control.event("actual-native-spawn", wrapperPid=os.getpid(), nativePid=child.pid)
    readers = [
        threading.Thread(target=control.drain, args=(stream, pipe), daemon=True)
        for stream, pipe in [("stdout", child.stdout), ("stderr", child.stderr)]
    ]
    for reader in readers:
        reader.start()
    selector = selectors.DefaultSelector()
    selector.register(0, selectors.EVENT_READ)
    buffer = bytearray()
    deadline = time.monotonic() + args.timeout_seconds
    timed_out = False
    killed = False
    try:
        while not control.stdin_complete or child.poll() is None:
            if time.monotonic() >= deadline:
                timed_out = True
                raise TimeoutError("actual native/control stdin did not terminate before deadline")
            for _, _ in selector.select(0.05):
                data = os.read(0, 65536)
                control.files["wrapper.stdin.bin"].write(data)
                if not data:
                    control.stdin_complete = True
                    selector.unregister(0)
                    if buffer:
                        raise ValueError("incomplete wrapper stdin LSP frame")
                    if not child.stdin.closed:
                        child.stdin.close()
                    break
                buffer.extend(data)
                for frame, message in frames(buffer):
                    control.forward(child, frame, message)
    except Exception as error:
        control.error(error)
        control.event("control-refusal", error=str(error))
        if timed_out and child.poll() is None:
            killed = True
            control.event("actual-native-kill-requested")
            child.kill()
    finally:
        selector.close()
        if not child.stdin.closed:
            child.stdin.close()
    try:
        native_status = child.wait(timeout=3 if killed else max(0.05, deadline - time.monotonic()))
    except subprocess.TimeoutExpired:
        timed_out = True
        killed = True
        control.event("actual-native-kill-requested")
        child.kill()
        try:
            native_status = child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            native_status = None
            control.error("actual native failed to reap after kill")
    for reader in readers:
        reader.join(timeout=3)
    if any(reader.is_alive() for reader in readers):
        control.error("native raw readers failed to drain")
    if not control.translated_exit or not control.shutdown_ack.is_set():
        control.error("missing acknowledged Shutdown/translated Exit")
    clean = (
        native_status == 0 and not timed_out and not killed and not control.errors
        and control.stdin_complete and control.stdout_complete and control.stderr_complete
    )
    if clean:
        control.emit("stderr", EOF_LINE, injected=True)
    wrapper_status = (
        124 if timed_out else 70 if control.errors or not clean else
        17 if args.mode == "nonzero" else native_status
    )
    if clean and args.mode in ("sigsegv", "timeout"):
        wrapper_status = None  # The parent owns the final signal/timeout receipt.
    control.event("actual-native-reaped", nativeExit=native_status, wrapperExit=wrapper_status)
    (args.control_dir / "native-child.txt").write_text(
        f"wrapper-pid={os.getpid()}\nnative-pid={child.pid}\n"
        f"native-exit={native_status}\nwrapper-exit={wrapper_status}\n"
    )
    receipt = {
        "schemaVersion": 1, "mode": args.mode, "cwd": os.getcwd(),
        "wrapperPid": os.getpid(), "parentPid": os.getppid(), "nativePid": child.pid,
        "wrapperSourceSha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "pythonExecutable": sys.executable, "pythonVersion": sys.version,
        "native": str(native), "nativeSha256": native_hash, "argv": native_args,
        "nativeExit": native_status, "nativeReaped": native_status is not None,
        "nativeSignal": -native_status if native_status is not None and native_status < 0 else None,
        "wrapperExit": wrapper_status,
        "timedOut": timed_out, "killRequested": killed, "errors": control.errors,
        "stdinComplete": control.stdin_complete, "stdoutComplete": control.stdout_complete,
        "stderrComplete": control.stderr_complete, "shutdownAcknowledged": control.shutdown_ack.is_set(),
        "exitTranslatedToNativeEof": control.translated_exit, "events": control.events,
        "shutdownFollowedByStdinEof": control.stdin_complete and control.shutdown_ack.is_set(),
        "injectedStderrOrigin": "wrapper; never attributed to native",
        "elapsedNs": time.monotonic_ns() - started,
    }
    for name, file in control.files.items():
        file.close()
    receipt["rawFiles"] = {
        name: {"bytes": path.stat().st_size, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        for name in control.files for path in [args.control_dir / name]
    }
    (args.control_dir / "control.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if clean and args.mode == "sigsegv":
        os.kill(os.getpid(), signal.SIGSEGV)
    if clean and args.mode == "timeout":
        time.sleep(30)  # The owning process must enforce its shutdown deadline.
    return wrapper_status


if __name__ == "__main__":
    raise SystemExit(main())
