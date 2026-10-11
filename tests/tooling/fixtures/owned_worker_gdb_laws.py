"""Exercise the actual debugger script with a deterministic GDB event adapter."""
import importlib.util
import io
import json
import os
from pathlib import Path
import signal
import sys
sys.dont_write_bytecode = True
import types
import unittest
from unittest.mock import patch

helpers = Path(sys.argv.pop(1))
sys.path.insert(0, str(helpers))


class Event:
    def __init__(self):
        self.handlers = []
    def connect(self, handler):
        self.handlers.append(handler)
    def disconnect(self, handler):
        self.handlers.remove(handler)
    def emit(self, event):
        for handler in self.handlers:
            handler(event)


class SignalEvent:
    def __init__(self, name):
        self.stop_signal = name


class FakeGDB:
    SignalEvent = SignalEvent
    def __init__(self, mode):
        self.mode, self.commands, self.text, self.continues = mode, [], [], 0
        self.events = types.SimpleNamespace(stop=Event(), exited=Event())
        def thread(number):
            actor = types.SimpleNamespace(num=number, ptid=(123, number, 0))
            actor.switch = lambda: setattr(self, "selected", actor)
            return actor
        self.original = thread(7)
        self.selected = self.original
        self.inferior = types.SimpleNamespace(pid=0, threads=lambda: [self.original, thread(11)])
    def write(self, body):
        self.text.append(body)
    def selected_inferior(self):
        return self.inferior
    def selected_thread(self):
        return self.selected
    def parse_and_eval(self, expression):
        if expression == "$pc": return 0x123
        if self.mode == "missing-siginfo": raise ValueError("private siginfo expression")
        return {"$_siginfo.si_signo": signal.SIGINT, "$_siginfo.si_code": -6,
                "$_siginfo._sifields._kill.si_pid": os.getpid() + (1 if self.mode == "original-interrupt" else 0),
                "$_siginfo._sifields._kill.si_uid": os.getuid()}[expression]
    def execute(self, command, to_string=False):
        self.commands.append(command)
        if command.startswith("attach "):
            self.inferior.pid = int(command.split()[1])
            if self.mode == "partial-attach-failure": raise RuntimeError("private startup arguments")
        elif command == "continue":
            self.continues += 1
            if self.mode == "cancel": raise KeyboardInterrupt()
            if self.mode == "normal" or self.continues == 2:
                self.events.exited.emit(types.SimpleNamespace(exit_code=0 if self.mode == "normal" else None))
                self.inferior.pid = 0
            else:
                self.events.stop.emit(SignalEvent("SIGINT" if self.mode in ["interrupt", "original-interrupt", "missing-siginfo"] else "SIGTERM" if self.mode == "other-signal" else "SIGSEGV"))
        elif command.startswith("signal "):
            self.events.exited.emit(types.SimpleNamespace(exit_code=None))
            self.inferior.pid = 0
        elif command == "bt 32":
            if self.mode == "backtrace-failure": raise ValueError("private secret")
            return "#0 0x123 in physical_provider ()\n"
        elif command == "info registers":
            return "registers-thread-" + str(self.selected.num)
        elif command == "detach":
            self.inferior.pid = 0
        return "bounded diagnostic identity\n"


class GDBLaws(unittest.TestCase):
    def run_fixture(self, mode, match=True, birth=True, ack="captured\n", cleanup_marker=False):
        adapter = FakeGDB(mode)
        with patch.dict(sys.modules, {"gdb": adapter}):
            spec = importlib.util.spec_from_file_location("owned_worker_gdb_law", helpers / "owned_worker_gdb.py")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
        actor = {"pid": 123, "startTicks": "456", "uid": 789, "executable": "/owned/node", "state": "T"}
        config = {"worker": {"actor": actor}, "leader": {"pid": 100}, "profile": {}, "root": "/owned"}
        signals, closed = [], []
        with patch.object(module, "state", return_value=actor if birth else dict(actor, startTicks="recycled")), patch.object(module, "select", return_value={"actor": actor} if match else None), patch.object(module.os, "pidfd_open", return_value=77, create=True), patch.object(module.os, "close", side_effect=closed.append), patch.object(module.signal, "pidfd_send_signal", side_effect=lambda fd, sig: signals.append((fd, sig)), create=True), patch.object(module.sys, "stdin", io.StringIO(ack)), patch.object(module.Path, "exists", return_value=cleanup_marker):
            try:
                module.run(config)
            except KeyboardInterrupt:
                self.assertEqual(mode, "cancel")
        packets = []
        for body in adapter.text:
            lines = body.splitlines()
            self.assertEqual(lines[0], module.BEGIN)
            self.assertEqual(lines[-1], module.END)
            packets.append(json.loads(lines[1]))
        self.assertEqual(adapter.events.stop.handlers, [])
        self.assertEqual(adapter.events.exited.handlers, [])
        self.assertNotIn("private secret", "".join(adapter.text))
        self.assertNotIn("private startup arguments", "".join(adapter.text))
        self.assertFalse(any("bt full" in command or "locals" in command or "args" in command or "generate-core" in command for command in adapter.commands))
        return adapter, packets, signals, closed

    def test_exact_owned_fault_is_marked_and_original_segv_is_passed(self):
        adapter, packets, signals, closed = self.run_fixture("fault")
        self.assertEqual([packet["status"] for packet in packets], ["attached", "SIGSEGV", "exited"])
        self.assertEqual(adapter.commands.count("continue"), 2)
        fault = next(packet for packet in packets if packet["status"] == "SIGSEGV")
        self.assertEqual(fault["faultingThread"]["number"], 7)
        self.assertEqual(fault["registers"], "registers-thread-7")
        self.assertEqual(adapter.selected.num, 7)
        self.assertIn((77, signal.SIGCONT), signals)
        self.assertEqual(closed, [77])

    def test_recycled_birth_or_wrong_actor_profile_refuses_before_attach(self):
        for birth, match in [(False, True), (True, False)]:
            adapter, packets, signals, closed = self.run_fixture("fault", match=match, birth=birth)
            self.assertFalse(any(command.startswith("attach ") for command in adapter.commands))
            self.assertFalse(any(sig == signal.SIGCONT for fd, sig in signals))
            self.assertEqual(packets[0]["status"], "unavailable")

    def test_partial_attach_failure_and_cancellation_attempt_detach(self):
        for mode in ["partial-attach-failure", "cancel"]:
            adapter, packets, signals, closed = self.run_fixture(mode)
            self.assertIn("detach", adapter.commands)
            self.assertEqual(closed, [77])

    def test_collection_failure_never_consumes_original_fault(self):
        adapter, packets, signals, closed = self.run_fixture("backtrace-failure")
        self.assertIn("signal SIGSEGV", adapter.commands)
        self.assertTrue(any(packet.get("gap") == "ValueError" for packet in packets))
        self.assertEqual(closed, [77])

    def test_missing_acknowledgement_passes_original_fault_and_marks_gap(self):
        adapter, packets, signals, closed = self.run_fixture("fault", ack="")
        self.assertEqual(adapter.commands.count("continue"), 2)
        self.assertTrue(any(packet.get("gap") == "MissingProviderAcknowledgement" for packet in packets))

    def test_other_original_signal_is_passed_and_normal_exit_has_no_fault(self):
        adapter, packets, signals, closed = self.run_fixture("other-signal")
        self.assertIn("signal SIGTERM", adapter.commands)
        self.assertNotIn("SIGSEGV", [packet["status"] for packet in packets])
        adapter, packets, signals, closed = self.run_fixture("normal")
        self.assertEqual([packet["status"] for packet in packets], ["attached", "exited"])
        self.assertEqual(adapter.commands.count("continue"), 1)

    def test_only_proven_diagnostic_interrupt_is_discarded_before_detach(self):
        for cleanup_marker, original in [(False, False), (True, False), (True, True)]:
            adapter, packets, signals, closed = self.run_fixture("original-interrupt" if original else "interrupt", cleanup_marker=cleanup_marker)
            discard = cleanup_marker and not original
            self.assertEqual("queue-signal 0" in adapter.commands, discard)
            self.assertEqual("signal SIGINT" in adapter.commands, not discard)
            if discard:
                self.assertLess(adapter.commands.index("queue-signal 0"), adapter.commands.index("detach"))

    def test_missing_siginfo_is_partial_and_does_not_guess_cleanup_signal_origin(self):
        adapter, packets, signals, closed = self.run_fixture("missing-siginfo", cleanup_marker=True)
        self.assertNotIn("queue-signal 0", adapter.commands)
        self.assertIn("signal SIGINT", adapter.commands)
        self.assertTrue(any(packet.get("gap") == "InterruptOriginValueError" for packet in packets))
        self.assertNotIn("private siginfo expression", json.dumps(packets))


if __name__ == "__main__":
    unittest.main()
