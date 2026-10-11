"""Controls-only execution never starts the original runner; provider gaps stay partial."""
import io
import importlib.util
import json
from pathlib import Path
import queue
import sys
sys.dont_write_bytecode = True
import tempfile
import unittest
from unittest.mock import patch

helpers = Path(sys.argv.pop(1))
sys.path.insert(0, str(helpers))
import owned_worker_runner as runner
from owned_worker_capture import write_json


class RunnerLaws(unittest.TestCase):
    def test_failed_live_control_drains_late_reap_packet_before_proven_resume_and_release(self):
        spec = importlib.util.spec_from_file_location("owned_live_cleanup_law", Path(__file__).with_name("owned_worker_live_laws.py"))
        live = importlib.util.module_from_spec(spec)
        with patch.object(sys, "argv", ["fixture", str(helpers)]): spec.loader.exec_module(live)
        for birth, tracer, reaped, released in [(True, 0, True, True), (False, 0, True, False), (True, 9, True, False), (True, 0, False, False)]:
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary); receipt = {}; packets = queue.Queue(16); calls = []
                class Worker:
                    events = []
                    def close(self): self.events = ["owned-worker-continued"]; calls.append("resume")
                    def check(self):
                        calls.append("birth")
                        if not birth: raise ValueError("private worker")
                    def tracer(self): calls.append("tracer"); return tracer
                class Child:
                    returncode = None
                    def poll(self): return self.returncode
                    def wait(self, timeout):
                        calls.append(("wait", timeout))
                        if not (directory / "release").exists() and self.returncode is None: raise runner.subprocess.TimeoutExpired("private argv", timeout)
                        self.returncode = self.returncode or 0
                        return self.returncode
                    def kill(self): calls.append("kill"); self.returncode = -9
                def late_cleanup(child, reader, outcome):
                    calls.append("debugger-reap")
                    packets.put({"status": "debugger-cleanup", "debuggerReaped": reaped})
                with patch.object(live, "close_gdb", late_cleanup):
                    live.cleanup_case(Child(), object(), object(), packets, Worker(), None, None, directory, directory, directory, receipt)
                self.assertEqual((directory / "release").exists(), released)
                self.assertEqual(receipt["gdbPackets"][0]["debuggerReaped"], reaped)
                self.assertLess(calls.index("debugger-reap"), calls.index("resume"))
                self.assertTrue(all(call[1] == 5 for call in calls if isinstance(call, tuple)))
                self.assertEqual("kill" in calls, not released)
                self.assertNotIn("private", json.dumps(receipt))

    def test_controls_only_never_invokes_original_runner_and_refuses_source_drift(self):
        for drift, failed, expected in [(False, False, 0), (True, False, 1), (False, True, 1)]:
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                def census(path, phase, receipt):
                    if phase == "after": receipt["sourceDrift"] = ["changed.ts"] if drift else []
                with patch.object(runner, "authenticate"), patch.object(runner, "source_census", census), patch.object(runner, "live_controls", side_effect=ValueError() if failed else None), patch.object(runner.subprocess, "Popen", side_effect=AssertionError("Original runner must not start")):
                    self.assertEqual(runner.controls_only(output), expected)
                receipt = json.loads((output / "public/controls-only-outcome.json").read_text())
                self.assertFalse(receipt["originalRunnerStarted"])
                self.assertEqual(receipt["controlReturnCode"], expected)

    def test_changed_missing_or_new_fault_provider_is_gap_but_original_signal_acknowledges(self):
        original = {"path": "/owned/provider", "sha256": "original", "inode": 1, "device": 1}
        for providers, gap in [([original], False), ([dict(original, sha256="changed")], True), ([], True), ([original, dict(original, path="/new/provider")], True)]:
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory); receipt = {}; packets = queue.Queue(16)
                write_json(output / "public/worker-before.json", {"providers": [original]})
                packets.put({"status": "SIGSEGV"})
                class Debugger:
                    stdin = io.BytesIO()
                    def poll(self): return None
                debugger = Debugger()
                with patch.object(runner, "physical_packet", return_value={"providers": providers}):
                    runner.drain_packets(packets, debugger, object(), object(), object(), output, output, receipt)
                self.assertEqual("faultProviderCustodyGap" in receipt, gap)
                self.assertEqual(debugger.stdin.getvalue(), b"captured\n")
                self.assertEqual(json.loads((output / "public/worker-fault.json").read_text())["providers"], providers)


if __name__ == "__main__":
    unittest.main()
