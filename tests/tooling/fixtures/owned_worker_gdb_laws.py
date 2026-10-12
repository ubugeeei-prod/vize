"""Exercise the actual debugger script with a deterministic GDB event adapter."""
import importlib.util
import hashlib
import io
import json
import os
from pathlib import Path
import signal
import sys
sys.dont_write_bytecode = True
import types
import tempfile
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
        self.policy = {name: {"stop": True, "print": True, "pass": True} for name in ["SIGSEGV", "SIGCONT", "SIGSTOP", "SIGINT", "SIGTERM"]}
        self.events = types.SimpleNamespace(stop=Event(), exited=Event(), cont=Event())
        def thread(number):
            actor = types.SimpleNamespace(num=number, ptid=(123, number, 0))
            actor.switch = lambda: setattr(self, "selected", actor)
            return actor
        self.original = thread(7)
        self.selected = self.original
        self.inferior = types.SimpleNamespace(pid=0, num=1, threads=lambda: [self.original, thread(11)])
    def write(self, body):
        self.text.append(body)
    def flush(self):
        self.flushed = len(self.text)
    def selected_inferior(self):
        return self.inferior
    def selected_thread(self):
        return self.selected
    def parse_and_eval(self, expression):
        if expression == "$pc": return 0x123
        if self.mode == "missing-siginfo": raise ValueError("private siginfo expression")
        return {"$_siginfo.si_signo": signal.SIGINT, "$_siginfo.si_code": -6,
                "$_siginfo._sifields._kill.si_pid": os.getpid() + (1 if self.mode in ["original-interrupt", "reraised-interrupt"] else 0),
                "$_siginfo._sifields._kill.si_uid": os.getuid()}[expression]
    def execute(self, command, to_string=False):
        self.commands.append(command)
        if command.startswith("handle "):
            policy = self.policy[command.split()[1]]
            for word in command.split()[2:]:
                if word == "stop": policy.update(stop=True, print=True)
                elif word == "nostop": policy["stop"] = False
                elif word == "noprint": policy.update(stop=False, print=False)
                elif word == "print": policy["print"] = True
                elif word == "pass": policy["pass"] = True
                elif word == "nopass": policy["pass"] = False
        elif command.startswith("info signals "):
            name = command.split()[-1]
            flags = ["Yes" if self.policy[name][key] else "No" for key in ["stop", "print", "pass"]]
            if self.mode == "policy-refusal" and name == "SIGSEGV": flags[0] = "No"
            if self.mode == "cont-policy-refusal" and name == "SIGCONT": flags[0] = "Yes"
            if self.mode == "postfault-policy-refusal" and self.continues and name == "SIGSEGV": flags[0] = "Yes"
            return "Signal Stop Print Pass to program Description\n" + name + " " + " ".join(flags) + " Signal description\n"
        elif command.startswith("attach "):
            self.inferior.pid = int(command.split()[1])
            if self.mode == "partial-attach-failure": raise RuntimeError("private startup arguments")
            if self.mode == "attach-continue-normal":
                start = len(self.text)
                self.events.cont.emit(types.SimpleNamespace())
                output = ''.join(self.text[start:])
                del self.text[start:]
                return output
        elif command == "continue":
            self.continues += 1
            if self.mode == "foreign-continued": self.inferior.pid += 1
            if self.mode == "postfault-exit" and self.continues == 2: self.inferior.pid = 0
            self.events.cont.emit(types.SimpleNamespace())
            if self.mode == "cancel": raise KeyboardInterrupt()
            if self.mode == "reraised-fault" and self.continues == 2 and self.policy['SIGSEGV']['stop']:
                self.events.stop.emit(SignalEvent('SIGSEGV'))
            elif self.mode in ["normal", "attach-continue-normal"] or self.continues == 2:
                self.events.exited.emit(types.SimpleNamespace(exit_code=0 if self.mode in ["normal", "attach-continue-normal"] else None, inferior=self.inferior))
                self.inferior.pid = 0
            else:
                self.events.stop.emit(SignalEvent("SIGINT" if self.mode in ["interrupt", "original-interrupt", "reraised-interrupt", "missing-siginfo"] else "SIGTERM" if self.mode == "other-signal" else "SIGSEGV"))
        elif command.startswith("signal "):
            self.events.cont.emit(types.SimpleNamespace())
            if self.mode == 'reraised-interrupt' and self.policy['SIGINT']['stop']:
                self.events.stop.emit(SignalEvent('SIGINT'))
                return 'private repeated signal stop\n'
            self.events.exited.emit(types.SimpleNamespace(exit_code=None, inferior=self.inferior))
            self.inferior.pid = 0
        elif command == "bt 32":
            if self.mode == "backtrace-failure": raise ValueError("private secret")
            return "#0 0x123 in physical_provider ()\n"
        elif command == "info registers":
            return "registers-thread-" + str(self.selected.num)
        elif command == "detach":
            self.events.exited.emit(types.SimpleNamespace(inferior=self.inferior))
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
        provider = {'executable': {'sha256': 'owned-debugger'}, 'version': {'value': '15.1'}}
        providers = [provider, {**provider, 'executable': {'sha256': 'changed'}} if mode == 'debugger-provider-drift' else provider]
        with patch.object(module, "debugger_identity", side_effect=providers, create=True), patch.object(module, 'private_text', return_value={'bytes': 0, 'truncated': False, 'uploaded': False}, create=True), patch.object(module, "state", return_value=actor if birth else dict(actor, startTicks="recycled")), patch.object(module, "select", return_value={"actor": actor} if match else None), patch.object(module.os, "pidfd_open", return_value=77, create=True), patch.object(module.os, "close", side_effect=closed.append), patch.object(module.signal, "pidfd_send_signal", side_effect=lambda fd, sig: signals.append((fd, sig)), create=True), patch.object(module.sys, "stdin", io.StringIO(ack)), patch.object(module.Path, "exists", return_value=cleanup_marker):
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
        self.assertEqual(adapter.events.cont.handlers, [])
        self.assertLessEqual(sum(packet.count('"status":"exited"') for packet in adapter.text), 1)
        self.assertEqual(adapter.flushed, len(adapter.text))
        self.assertNotIn("private secret", "".join(adapter.text))
        self.assertNotIn("private startup arguments", "".join(adapter.text))
        self.assertFalse(any("bt full" in command or "locals" in command or "args" in command or "generate-core" in command for command in adapter.commands))
        return adapter, packets, signals, closed

    def test_exact_owned_fault_is_marked_and_original_segv_is_passed(self):
        adapter, packets, signals, closed = self.run_fixture("fault")
        self.assertEqual([packet["status"] for packet in packets if packet["status"] != "debugger-phase"], ["attached", "SIGSEGV", "exited"])
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
            self.assertEqual([packet['gap'] for packet in packets if packet['status'] == 'unavailable'], ['ValueError'])

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
        self.assertEqual([packet["status"] for packet in packets if packet["status"] != "debugger-phase"], ["attached", "exited"])
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

    def test_noprint_implies_nostop_and_actual_policy_must_refuse_before_attach(self):
        adapter = FakeGDB("fault")
        adapter.execute("handle SIGSEGV stop noprint pass")
        self.assertFalse(adapter.policy["SIGSEGV"]["stop"])
        for mode in ["policy-refusal", "cont-policy-refusal"]:
            adapter, packets, signals, closed = self.run_fixture(mode)
            self.assertFalse(any(command.startswith("attach ") for command in adapter.commands))
            self.assertEqual(signals, [])
            self.assertEqual(closed, [])
            self.assertTrue(any(packet.get("gap") == "ValueError" for packet in packets))

    def test_changed_continued_inferior_has_no_fault_credit_or_forwarded_signal(self):
        adapter, packets, signals, closed = self.run_fixture("foreign-continued")
        self.assertNotIn("SIGSEGV", [packet["status"] for packet in packets])
        self.assertNotIn("signal SIGSEGV", adapter.commands)
        self.assertNotIn("queue-signal 0", adapter.commands)
        self.assertFalse(any(packet.get("phase") == "inferior-running" for packet in packets))
        self.assertEqual(closed, [77])

    def test_readiness_is_once_and_does_not_reinspect_expected_postfault_death(self):
        adapter, packets, signals, closed = self.run_fixture("postfault-exit")
        self.assertEqual(sum(packet.get("phase") == "inferior-running" for packet in packets), 1)
        self.assertFalse(any(packet.get("gap", "").startswith("ContinuedActor") for packet in packets))
        self.assertEqual([packet["status"] for packet in packets if packet["status"] != "debugger-phase"], ["attached", "SIGSEGV", "exited"])

    def test_attach_time_cont_cannot_consume_owned_first_resume_readiness(self):
        adapter, packets, signals, closed = self.run_fixture("attach-continue-normal")
        self.assertEqual(sum(packet.get("phase") == "inferior-running" for packet in packets), 1)
        self.assertFalse(any(packet.get("gap", "").startswith("ContinuedActor") for packet in packets))
        observations = next(packet for packet in packets if packet.get("phase") == "callback-observations")
        self.assertEqual({key: observations[key] for key in ['duringAttach', 'afterAttach', 'ownedReady']}, {'duringAttach': 1, 'afterAttach': 1, 'ownedReady': True})
        self.assertEqual([packet['status'] for packet in packets if packet['status'] != 'debugger-phase'], ['attached', 'exited'])

    def test_reraised_original_fault_passes_after_one_capture_to_actual_exit(self):
        adapter, packets, signals, closed = self.run_fixture('reraised-fault')
        self.assertEqual([packet['status'] for packet in packets if packet['status'] != 'debugger-phase'], ['attached', 'SIGSEGV', 'exited'])
        self.assertEqual(adapter.commands.count('continue'), 2)
        self.assertEqual(adapter.policy['SIGSEGV'], {'stop': False, 'print': False, 'pass': True})
        phase = next(packet for packet in packets if packet.get('phase') == 'original-signal-onward-policy')
        self.assertEqual(phase['signal'], 'SIGSEGV')

    def test_genuine_reraised_interrupt_is_forwarded_without_diagnostic_discard(self):
        adapter, packets, signals, closed = self.run_fixture('reraised-interrupt', cleanup_marker=True)
        self.assertNotIn('queue-signal 0', adapter.commands)
        self.assertIn('signal SIGINT', adapter.commands)
        self.assertEqual(sum(packet['status'] == 'exited' for packet in packets), 1)
        self.assertFalse(any(packet['status'] == 'detached' for packet in packets))
        self.assertEqual(adapter.policy['SIGINT'], {'stop': False, 'print': False, 'pass': True})

    def test_postfault_onward_policy_refusal_has_no_exit_or_forwarding_credit(self):
        adapter, packets, signals, closed = self.run_fixture('postfault-policy-refusal')
        self.assertEqual(adapter.commands.count('continue'), 1)
        self.assertNotIn('signal SIGSEGV', adapter.commands)
        self.assertFalse(any(packet['status'] == 'exited' for packet in packets))
        self.assertFalse(any(packet.get('phase') == 'original-signal-onward-policy' for packet in packets))
        self.assertTrue(any(packet.get('gap') == 'ValueError' for packet in packets))

    def test_debugger_body_drift_is_explicit_without_consuming_original_fault(self):
        adapter, packets, signals, closed = self.run_fixture('debugger-provider-drift')
        self.assertTrue(any(packet.get('gap') == 'DebuggerProviderValueError' for packet in packets))
        self.assertEqual(adapter.commands.count('continue'), 2)
        self.assertEqual(sum(packet['status'] == 'SIGSEGV' for packet in packets), 1)

    def test_captured_attach_text_remains_private_and_bounded(self):
        with patch.dict(sys.modules, {'gdb': FakeGDB('normal')}):
            spec = importlib.util.spec_from_file_location('owned_worker_private_text_law', helpers / 'owned_worker_gdb.py')
            module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as directory, patch.object(module, '__file__', str(Path(directory) / 'owned_worker_gdb.py')):
            body = 'private argv/env ' + 'x' * 140000
            receipt = module.private_text('attach-private.txt', body); path = Path(directory) / 'attach-private.txt'
            self.assertEqual(path.read_bytes(), body.encode()[:131072]); self.assertEqual(path.stat().st_mode & 0o777, 0o600)
            self.assertTrue(receipt['truncated']); self.assertFalse(receipt['uploaded'])
            self.assertEqual(receipt['sha256'], hashlib.sha256(body.encode()).hexdigest()); self.assertNotIn('private argv/env', json.dumps(receipt))


if __name__ == "__main__":
    unittest.main()
