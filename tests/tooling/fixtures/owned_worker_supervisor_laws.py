"""Actual supervisor cleanup logic under deterministic owned-child failures."""
import io
import json
from pathlib import Path
import signal
import subprocess
import sys
sys.dont_write_bytecode = True
import tempfile
import types
import unittest
from unittest.mock import patch

helpers = Path(sys.argv.pop(1))
sys.path.insert(0, str(helpers))
import owned_worker_supervisor as supervisor


class Child:
    def __init__(self, failures):
        self.failures, self.calls, self.returncode = failures, [], None
    def poll(self):
        return self.returncode
    def send_signal(self, number):
        self.calls.append(("signal", number))
    def terminate(self):
        self.calls.append(("terminate",))
    def kill(self):
        self.calls.append(("kill",))
    def wait(self, timeout=None):
        self.calls.append(("wait", timeout))
        if self.failures:
            self.failures -= 1
            raise subprocess.TimeoutExpired("private debugger argv", timeout)
        self.returncode = 0
        return self.returncode


class SupervisorLaws(unittest.TestCase):
    def test_graceful_debugger_interrupt_is_reaped_before_worker_cleanup(self):
        child, receipt = Child(0), {}
        supervisor.reap_debugger(child, receipt)
        self.assertEqual(child.calls[0], ("signal", signal.SIGINT))
        self.assertTrue(receipt["debuggerReaped"])
        self.assertFalse(receipt.get("forcedDebuggerExit", False))

    def test_forced_exit_targets_actual_unreaped_child_then_reaps(self):
        child, receipt = Child(2), {}
        supervisor.reap_debugger(child, receipt)
        self.assertEqual([call[0] for call in child.calls], ["signal", "wait", "terminate", "wait", "kill", "wait", "wait"])
        self.assertTrue(receipt["forcedDebuggerExit"] and receipt["debuggerReaped"])
        self.assertEqual([(item["signal"], item["wait"]) for item in receipt["cleanupSignalAttempts"]], [("SIGINT", "timeout"), ("SIGTERM", "timeout"), ("SIGKILL", "reaped")])
        self.assertNotIn("private", json.dumps(receipt))

    def test_already_exited_debugger_has_no_signal_or_pid_lookup(self):
        child, receipt = Child(0), {}
        child.returncode = 0
        supervisor.reap_debugger(child, receipt)
        self.assertEqual(child.calls, [("wait", 0)])

    def test_all_cleanup_timeouts_refuse_reap_credit_without_unbounded_wait(self):
        child, receipt = Child(4), {}
        with self.assertRaises(subprocess.TimeoutExpired): supervisor.reap_debugger(child, receipt)
        self.assertTrue(receipt["forcedDebuggerExit"])
        self.assertNotIn("debuggerReaped", receipt)
        self.assertTrue(all(call[1] is not None for call in child.calls if call[0] == "wait"))

    def test_invalid_privileged_owner_refuses_without_child_or_worker(self):
        config = {"worker": {"actor": {"uid": 100}}}
        published = []
        with patch.object(supervisor.os, "geteuid", return_value=100), patch.object(supervisor, "publish", side_effect=published.append), patch.object(supervisor.subprocess, "Popen") as spawn:
            self.assertEqual(supervisor.supervise(config, Path("/unused")), 1)
            spawn.assert_not_called()
        self.assertEqual(published[0]["gap"], "ValueError")
        self.assertFalse(published[0]["debuggerReaped"])

    def test_input_protocol_never_forwards_unknown_private_content(self):
        import queue
        for text, expected in [("captured\n", ["captured", "detach"]), ("unknown secret\n", ["detach"]), ("", ["detach"])]:
            packets = queue.Queue(16)
            supervisor.input_reader(io.StringIO(text), packets)
            self.assertEqual(list(packets.queue), expected)

    def test_debugger_error_projection_excludes_messages_foreign_paths_and_context(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for name in ["owned_worker_gdb.py", "owned_worker_identity.py", "owned_worker_capture.py", "owned_worker_supervisor.py"]:
                (directory / name).write_text("reviewed source")
            text = ('Traceback (most recent call last):\n  File "' + str(directory / "owned_worker_gdb.py") + '", line 17, in private_context\n'
                    '  File "/foreign/owned_worker_gdb.py", line 1\n    private argv/environment\nNameError: private secret\n')
            receipt = {}
            supervisor.debugger_errors(io.BytesIO(text.encode()), directory, receipt)
            packet = receipt["debuggerStderr"]
            self.assertEqual(packet["bytes"], len(text.encode()))
            self.assertEqual(packet["recognizedErrorTypes"], ["NameError"])
            self.assertEqual([(frame["helper"], frame["line"]) for frame in packet["recognizedHelperFrames"]], [("owned_worker_gdb.py", 17)])
            self.assertNotIn("private", json.dumps(packet))
            self.assertNotIn("/foreign", json.dumps(packet))
            broken = {}
            supervisor.debugger_error_reader(io.BytesIO(b"secret"), directory / "missing", broken)
            self.assertEqual(broken["gap"], "DebuggerErrorReaderFileNotFoundError")


if __name__ == "__main__":
    unittest.main()
