"""Meaningful actor, lifecycle, privacy and original-runner refusal controls."""
import hashlib
import io
import json
import os
from pathlib import Path
import queue
import shutil
import signal
import subprocess
import sys
sys.dont_write_bytecode = True
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, sys.argv.pop(1))
import owned_worker_capture as capture
import owned_worker_identity as identity
import owned_worker_runner as runner


class ActorFixture:
    def __init__(self, directory):
        self.root = Path(directory).resolve()
        self.proc = self.root / "proc"
        self.proc.mkdir()
        package = self.root / identity.PACKAGE
        package.mkdir(parents=True)
        target = self.root / identity.TARGET
        target.parent.mkdir()
        target.write_text("authored original worker source\n")
        (package / "package.json").write_text(json.dumps({"scripts": {"test": identity.RECIPE}}))
        (package / "tsconfig.json").write_text('{"compilerOptions":{"module":"NodeNext"}}')
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(["git", "add", identity.PACKAGE], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Law", "-c", "user.email=law@example.invalid", "commit", "-qm", "test: source"], cwd=self.root, check=True)
        node = self.root / "node"
        header = bytearray(64); header[:6] = b"\x7fELF\x02\x01"
        header[16:20] = b"\x03\0\x3e\0"
        node.write_bytes(header)
        dist = self.root / "node_modules/tsx/dist"
        dist.mkdir(parents=True)
        for name in ["preflight.cjs", "loader.mjs"]:
            (dist / name).write_text("physical " + name)
        argv = [str(node), "--require", str(dist / "preflight.cjs"), "--import", (dist / "loader.mjs").as_uri(), str(Path(identity.TARGET).relative_to(identity.PACKAGE))]
        self.profile = {"privateArgv": argv, "executable": identity.file_identity(node),
                        "executableElf": identity.elf_identity(node),
                        "nodeOptionsBinding": {"bytes": len(identity.NODE_OPTIONS), "sha256": hashlib.sha256(identity.NODE_OPTIONS.encode()).hexdigest()},
                        "target": identity.target_identity(self.root),
                        "preloads": identity.preload_files(argv, self.root),
                        "configuration": [identity.file_identity(package / "package.json"), identity.file_identity(package / "tsconfig.json")]}
        self.birth(100, 1, "1000", node)
        self.birth(101, 100, "1001", node)
        self.birth(102, 101, "1002", node)
        self.leader = identity.state(100, self.proc)
        self.worker = self.proc / "102"
        (self.worker / "cmdline").write_bytes(b"\0".join(arg.encode() for arg in argv) + b"\0")
        (self.worker / "environ").write_bytes(b"PRIVATE_SECRET=never-publish\0NODE_TEST_CONTEXT=child-v8\0NODE_OPTIONS=" + identity.NODE_OPTIONS.encode() + b"\0")
        (self.worker / "cwd").symlink_to(package)

    def birth(self, pid, parent, start, executable):
        directory = self.proc / str(pid)
        directory.mkdir(exist_ok=True)
        fields = ["0"] * 24
        fields[0], fields[1], fields[19] = "S", str(parent), start
        (directory / "stat").write_text(str(pid) + " (node (private)) " + " ".join(fields))
        link = directory / "exe"
        if link.is_symlink():
            link.unlink()
        link.symlink_to(executable)

    def select(self, pid=102, owner_uid=None):
        return identity.select(pid, self.leader, self.profile, self.root, self.proc, owner_uid)


class ActorLaws(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.fixture = ActorFixture(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def test_exact_worker_has_closed_public_actor_proof(self):
        packet = self.fixture.select()
        self.assertTrue(packet["exactPrivateArgvMatch"] and packet["isolatedTestWorker"])
        self.assertNotIn("PRIVATE_SECRET", json.dumps(packet))
        self.assertNotIn("privateArgv", packet)
        self.assertEqual([item["pid"] for item in packet["ancestry"]], [102, 101])

    def test_driver_substring_and_extra_argument_never_select(self):
        f = self.fixture
        with self.assertRaises(ValueError):
            f.select(100)
        original = (f.worker / "cmdline").read_bytes()
        for body in [original + b"extra\0", original.replace(b"variant-bindings-http.test.ts", b"variant-bindings-http.test.ts.extra"), original[:-1]]:
            (f.worker / "cmdline").write_bytes(body)
            try:
                self.assertIsNone(f.select())
            except ValueError:
                pass

    def test_actual_worker_marker_and_foreign_owner_cwd_refuse(self):
        f = self.fixture
        for body in [b"NODE_TEST_CONTEXT=child\0", b"NODE_TEST_CONTEXT=child-v8\0NODE_TEST_CONTEXT=child-v8\0", b"NODE_TEST_CONTEXT=child-v8\0NODE_TEST_CONTEXT=other\0", b"PRIVATE_SECRET=never-publish\0"]:
            (f.worker / "environ").write_bytes(body)
            with self.assertRaises(ValueError):
                f.select()
        (f.worker / "environ").write_bytes(b"NODE_TEST_CONTEXT=child-v8\0NODE_OPTIONS=" + identity.NODE_OPTIONS.encode() + b"\0")
        with self.assertRaises(ValueError):
            f.select(owner_uid=os.getuid() + 1)
        (f.worker / "cwd").unlink()
        (f.worker / "cwd").symlink_to(f.root)
        with self.assertRaises(ValueError):
            f.select()

    def test_changed_source_node_preload_configuration_and_parent_birth_refuse(self):
        f = self.fixture
        for filename in [f.profile["target"]["path"], f.profile["executable"]["path"], f.profile["preloads"][0]["path"], f.profile["configuration"][1]["path"]]:
            path = Path(filename); original = path.read_bytes(); stamp = path.stat()
            path.write_bytes(original + b"changed")
            with self.assertRaises(ValueError):
                f.select()
            path.write_bytes(original); os.utime(path, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
        f.birth(100, 1, "recycled", Path(f.profile["executable"]["path"]))
        with self.assertRaises(ValueError):
            f.select()

    def test_absent_changed_or_duplicate_private_node_options_refuse(self):
        f = self.fixture; expected = identity.NODE_OPTIONS.encode()
        for options in [b"", b"NODE_OPTIONS=changed\0", b"NODE_OPTIONS=" + expected + b"\0NODE_OPTIONS=" + expected + b"\0"]:
            (f.worker / "environ").write_bytes(b"NODE_TEST_CONTEXT=child-v8\0" + options)
            with self.assertRaises(ValueError): f.select()

    def test_reparented_or_replaced_worker_identity_refuses(self):
        f = self.fixture
        f.birth(102, 1, "1002", Path(f.profile["executable"]["path"]))
        with self.assertRaises(ValueError):
            f.select()
        f.birth(102, 101, "1002", f.root / "unknown-node")
        with self.assertRaises(ValueError):
            f.select()

    def test_non_elf_wrong_machine_and_mapped_device_or_inode_refuse(self):
        f = self.fixture; path = Path(f.profile["executable"]["path"])
        original = path.read_bytes()
        for body in [b"not ELF", original[:18] + b"\xb7\0" + original[20:]]:
            path.write_bytes(body)
            with self.assertRaises(ValueError): identity.elf_identity(path)
        path.write_bytes(original)
        physical = path.stat(); device = format(os.major(physical.st_dev), "x") + ":" + format(os.minor(physical.st_dev), "x")
        maps = f.worker / "maps"
        valid = "0001-0002 r-xp 0000 " + device + " " + str(physical.st_ino) + " " + str(path) + "\n"
        maps.write_text(valid)
        self.assertEqual(identity.providers(102, f.proc)["providers"][0]["sha256"], identity.sha256(path))
        for body in [valid.replace(device, "ff:ff"), valid.replace(str(physical.st_ino), str(physical.st_ino + 1))]:
            maps.write_text(body)
            with self.assertRaises(ValueError): identity.providers(102, f.proc)

    def test_provider_changes_during_hash_refuse(self):
        path = Path(self.fixture.profile["executable"]["path"])
        original_hash = identity.sha256
        def changed(filename):
            value = original_hash(filename)
            Path(filename).write_bytes(Path(filename).read_bytes() + b"changed")
            return value
        with patch.object(identity, "sha256", changed), self.assertRaises(ValueError):
            identity.file_identity(path)

    def test_hash_cost_is_aggregate_outside_identity_equality(self):
        path = Path(self.fixture.profile["executable"]["path"])
        before = dict(identity.HASH_METRICS); first = identity.file_identity(path); second = identity.file_identity(path)
        self.assertEqual(first, second)
        self.assertEqual(identity.HASH_METRICS["calls"] - before["calls"], 2)
        self.assertEqual(identity.HASH_METRICS["bytes"] - before["bytes"], 2 * path.stat().st_size)
        self.assertGreaterEqual(identity.HASH_METRICS["seconds"], before["seconds"])


class LifecycleLaws(unittest.TestCase):
    def fixture(self, states=None, fail=None):
        actor = {"pid": 10, "startTicks": "42", "uid": 100, "executable": "/owned/node", "state": "T"}
        calls, closed = [], []
        class Ops:
            def pidfd_send_signal(self, fd, sig):
                calls.append((fd, sig))
                if fail and sig == fail:
                    raise ProcessLookupError()
        read = lambda pid: dict(actor)
        if states:
            read = lambda pid: states.pop(0) if len(states) > 1 else states[0]
        worker = capture.Worker(actor, Ops(), read, lambda pid: 99, closed.append, lambda: 0)
        return worker, actor, calls, closed

    def test_pidfd_only_stop_continue_and_idempotent_cleanup(self):
        worker, actor, calls, closed = self.fixture()
        worker.stop(); worker.close(); worker.close()
        self.assertEqual(calls, [(99, 0), (99, signal.SIGSTOP), (99, 0), (99, signal.SIGCONT)])
        self.assertEqual(closed, [99])

    def test_recycled_birth_before_stop_or_resume_never_signals_replacement(self):
        for phase in ["stop", "resume"]:
            worker, actor, calls, closed = self.fixture()
            if phase == "resume": worker.stop()
            worker.read_state = lambda pid: dict(actor, startTicks="43")
            if phase == "stop":
                with self.assertRaises(ValueError): worker.stop()
            worker.close()
            self.assertNotIn((99, signal.SIGCONT), calls)
            self.assertEqual(closed, [99])

    def test_birth_replacement_between_open_and_recheck_closes_fd_without_stop(self):
        actor = {"pid": 10, "startTicks": "42", "uid": 100, "executable": "/owned/node", "state": "T"}
        closed = []
        with self.assertRaises(ValueError):
            capture.Worker(actor, object(), lambda pid: actor if not closed else dict(actor, startTicks="43"), lambda pid: closed.append("opened") or 99, closed.append)
        self.assertEqual(closed, ["opened", 99])

    def test_esrch_is_an_explicit_gap_not_pid_lookup_fallback(self):
        worker, actor, calls, closed = self.fixture()
        worker.stop()
        worker.ops.pidfd_send_signal = lambda fd, sig: (_ for _ in ()).throw(ProcessLookupError())
        worker.close()
        self.assertIn("ESRCH-observation-gap", worker.events)
        self.assertEqual(closed, [99])

    def test_unknown_active_tracer_refuses_resume_until_reaped(self):
        worker, actor, calls, closed = self.fixture()
        worker.stop(); worker.read_tracer = lambda: 999; worker.close()
        self.assertNotIn((99, signal.SIGCONT), calls)
        self.assertIn("resume-refused-ValueError", worker.events)

    @unittest.skipUnless(sys.platform == "linux", "real pidfd law requires Linux")
    def test_real_owned_child_stops_resumes_and_exits_without_orphan(self):
        child = subprocess.Popen([sys.executable, "-c", "import sys;sys.stdin.read(1)"], stdin=subprocess.PIPE)
        worker = None
        try:
            actor = identity.state(child.pid)
            worker = capture.Worker(actor)
            worker.stop()
            self.assertIn(identity.state(child.pid)["state"], ["T", "t"])
            worker.close()
            child.stdin.write(b"x"); child.stdin.flush()
            self.assertEqual(child.wait(timeout=5), 0)
            self.assertIn("owned-worker-continued", worker.events)
        finally:
            if worker: worker.close()
            if child.poll() is None: child.kill(); child.wait()


class PrivacyAndRunnerLaws(unittest.TestCase):
    def test_reader_error_is_an_explicit_gap_with_no_text(self):
        class BrokenReader:
            def readline(self, count): raise OSError("private stream arguments")
        receipt = {}; capture.marker_reader(BrokenReader(), queue.Queue(16), receipt)
        self.assertEqual(receipt, {"readerGap": "OSError"})

    def test_fresh_git_stage_and_whole_source_census_leave_no_bytecode(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory).resolve(); root = base / "source"; root.mkdir()
            owned = Path(capture.__file__).parent
            entry = root / "tools/support/compat/github"; entry.mkdir(parents=True)
            legacy = root / ".github/actions/test-js-packages-with-history"; legacy.mkdir(parents=True)
            actual_legacy = owned.parents[3] / ".github/actions/test-js-packages-with-history"
            for name in capture.FILES: shutil.copyfile(owned / name, entry / name)
            for name in capture.SOURCE_CENSUS: shutil.copyfile(actual_legacy / name, legacy / name)
            live = root / "tests/tooling/fixtures"; live.mkdir(parents=True)
            shutil.copyfile(owned.parents[3] / "tests/tooling/fixtures" / capture.LIVE_LAWS, live / capture.LIVE_LAWS)
            authored = root / "source.txt"; authored.write_text("whole authored source")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(["git", "-c", "user.name=Law", "-c", "user.email=law@example.invalid", "commit", "-qm", "test: custody"], cwd=root, check=True)
            head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
            environment = dict(os.environ, RUNNER_TEMP=str(base), GITHUB_OUTPUT=str(base / "outputs"), GITHUB_RUN_ID="123", GITHUB_RUN_ATTEMPT="1", GITHUB_JOB="source-law")
            result = subprocess.run([sys.executable, str(entry / "owned_worker_capture.py"), "stage", head], cwd=root, env=environment, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(list(root.rglob("__pycache__")))
            self.assertEqual(subprocess.check_output(["git", "status", "--porcelain"], cwd=root), b"")
            output = base / "owned-worker-fault-123-1-source-law"
            current = Path.cwd()
            try:
                os.chdir(root)
                receipt = {}; runner.source_census(output, "before", receipt)
                authored.write_text("whole changed physical source")
                runner.source_census(output, "after", receipt)
            finally:
                os.chdir(current)
            before = json.loads((output / "public/source-before.json").read_text())
            after = json.loads((output / "public/source-after.json").read_text())
            self.assertEqual(len(before["files"]), len(capture.FILES) + len(capture.SOURCE_CENSUS) + 2)
            values = lambda packet: next(item["sha256"] for item in packet["files"] if item["file"] == "source.txt")
            self.assertNotEqual(values(before), values(after))
            self.assertNotEqual(before["diffSha256"], after["diffSha256"])
            self.assertEqual(receipt["sourceDrift"], ["source.txt"])
            self.assertFalse(receipt["sourceCensusAfter"]["wholePhysicalBodiesEqual"])
            self.assertFalse(list(root.rglob("__pycache__")))

    def test_markers_only_bounded_and_missing_or_malformed_markers_refuse(self):
        for text, count, gap in [
            (b"private argv/secret\n", 0, None),
            (capture.BEGIN + b'{"status":"attached","pid":1}\n' + capture.END, 1, None),
            (capture.BEGIN + b'{"status":"attached","argv":"secret"}\n' + capture.END, 0, "MalformedOrExcessPacket"),
            (capture.BEGIN + b'{"status":"attached"}\n', 0, "FinalPartialMarker"),
            (capture.BEGIN + b"x" * 150000 + b"\n" + capture.END, 0, "DiagnosticTextExceedsBound")]:
            packets, receipt = queue.Queue(16), {}
            capture.marker_reader(io.BytesIO(b"private startup\n" + text), packets, receipt)
            self.assertEqual(packets.qsize(), count)
            self.assertEqual(receipt.get("markerGap"), gap)
            self.assertNotIn("secret", json.dumps(receipt))
            self.assertEqual(receipt["gdbStreamAuthority"]["sha256"], hashlib.sha256(b"private startup\n" + text).hexdigest())

    def test_setup_failure_preserves_once_exact_runner_environment_output_and_status(self):
        for status in [0, 9, -signal.SIGTERM, -signal.SIGKILL]:
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                count = output / "calls.jsonl"
                source = "import json,os,signal,sys;open(sys.argv[1],'a').write(json.dumps({'argv':sys.argv[1:],'env':dict(os.environ),'cwd':os.getcwd()})+'\\n');print('original stdout');print('original stderr',file=sys.stderr);" + ("os.kill(os.getpid()," + str(-status) + ")" if status < 0 else "sys.exit(" + str(status) + ")")
                command = [sys.executable, "-c", source, str(count), "quoted ' and newline\n"]
                with patch.object(runner, "authenticate", side_effect=PermissionError("typed setup gap")):
                    actual = runner.observe(command, output)
                self.assertEqual(actual, status)
                calls = [json.loads(line) for line in count.read_text().splitlines()]
                self.assertEqual(len(calls), 1)
                self.assertTrue(calls[0]["argv"] == command[3:], "Original private argument vector changed")
                self.assertTrue(calls[0]["env"] == dict(os.environ), "Original private environment changed")
                self.assertEqual(calls[0]["cwd"], str(Path.cwd().resolve()))
                receipt = json.loads((output / "public/outcome.json").read_text())
                self.assertEqual(receipt["originalReturnCode"], status)
                self.assertEqual(receipt["setupGap"], "PermissionError")
                self.assertNotIn("argv", receipt)

    def test_private_transcript_is_bounded_and_mode600_without_public_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'stdout-private.txt'; receipt = {}; packets = queue.Queue(16)
            body = b'private argv/env\n' + b'x' * 140000
            capture.marker_reader(io.BytesIO(body), packets, receipt, path)
            self.assertEqual(path.read_bytes(), body[:131072]); self.assertEqual(path.stat().st_mode & 0o777, 0o600)
            self.assertTrue(receipt['privateTranscript']['truncated']); self.assertFalse(receipt['privateTranscript']['uploaded'])
            self.assertEqual(receipt['privateTranscript']['sha256'], hashlib.sha256(body[:131072]).hexdigest())
            self.assertNotIn('private argv/env', json.dumps(receipt)); self.assertEqual(packets.qsize(), 0)


if __name__ == "__main__":
    unittest.main()
