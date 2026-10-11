"""Explicit Linux controls for the actual GDB/sudo/pidfd cleanup protocol."""
import json
import os
from pathlib import Path
import shutil
import resource
import signal
import subprocess
import sys
sys.dont_write_bytecode = True
import tempfile
import time

output = Path(sys.argv[1]).resolve()
sys.path.insert(0, str(output))
from owned_worker_capture import FILES, Worker, close_gdb, start_gdb, write_json
from owned_worker_identity import PACKAGE, RECIPE, TARGET, file_identity, prepare_profile, select, state
from owned_worker_runner import drain_packets, physical_packet


def checked(command, cwd):
    subprocess.run(command, cwd=cwd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True, timeout=10)


def wait_for(predicate, seconds=10):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(.01)
    raise TimeoutError("Owned live control did not reach its expected phase")


def disable_core():
    _, hard = resource.getrlimit(resource.RLIMIT_CORE)
    resource.setrlimit(resource.RLIMIT_CORE, (0, hard))


def run_case(root, stage, directory, mode):
    controls = directory / "control"
    controls.mkdir()
    # This is an authored toy Git source, never a change to the original SDK
    # target, test inputs, assertion body or client deadline.
    source = root / TARGET
    body = """import {test} from 'node:test';import {watch,existsSync,writeFileSync,readFileSync} from 'node:fs';
void test('owned diagnostic control',async()=>{const directory=process.cwd()+'/../../../control-current';
writeFileSync(directory+'/actual-worker-private.json',JSON.stringify({argv:readFileSync('/proc/self/cmdline','utf8').split('\\0').slice(0,-1),testWorker:process.env.NODE_TEST_CONTEXT==='child-v8'}),{flag:'wx',mode:0o600});
const watcher=watch(directory);await new Promise(resolve=>{watcher.on('change',()=>{if(existsSync(directory+'/release'))resolve();});writeFileSync(directory+'/ready','ready');});
watcher.close();if(readFileSync(directory+'/release','utf8')==='fault')process.kill(process.pid,'SIGSEGV');});
"""
    source.write_text(body)
    checked(["git", "add", TARGET], root)
    checked(["git", "-c", "user.name=Law", "-c", "user.email=law@example.invalid", "commit", "--allow-empty", "-qm", "test: live worker"], root)
    link = root / "control-current"
    if link.is_symlink(): link.unlink()
    link.symlink_to(controls)
    probe = stage / "worker-profile.test.mjs"
    shutil.copyfile(output / "owned_worker_probe.mjs", probe)
    profile = prepare_profile(root, probe, stage / "probe-private.json", dict(os.environ))
    child = subprocess.Popen(["vp", "exec", "tsx", "--test", "--test-reporter=spec", "--test-reporter=tap", "--test-reporter-destination=stdout", "--test-reporter-destination=stderr", str(Path(TARGET).relative_to(PACKAGE))], cwd=root / PACKAGE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, preexec_fn=disable_core)
    worker, debugger, reader, packets = None, None, None, None
    receipt = {"control": mode, "qualification": "synthetic control only"}
    try:
        wait_for(lambda: (controls / "ready").exists())
        leader = state(child.pid)
        def find():
            for entry in Path("/proc").iterdir():
                if not entry.name.isdigit(): continue
                try:
                    actor = select(int(entry.name), leader, profile, root)
                    if actor is not None: return actor
                except (OSError, ValueError, IndexError): pass
            return None
        selected = wait_for(find)
        wait_for(lambda: (controls / "ready").exists())
        private = controls / "actual-worker-private.json"
        actual_profile = json.loads(private.read_text())
        if actual_profile["argv"] != profile["privateArgv"] or actual_profile["testWorker"] is not True:
            raise ValueError("Actual relative target worker differs from the frozen private probe")
        receipt["actualPrivateProfileMatch"] = True
        receipt["privateProfileAuthority"] = {key: value for key, value in file_identity(private).items() if key in ["bytes", "sha256"]}
        worker = Worker(selected["actor"]); worker.stop()
        selected = select(worker.identity["pid"], leader, profile, root)
        write_json(stage / "public/worker-before.json", physical_packet(worker, leader, profile, root))
        debugger, packets, reader = start_gdb(stage, selected, leader, profile, root, receipt)
        def attached():
            drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
            return any(packet["status"] == "attached" for packet in receipt.get("gdbPackets", []))
        wait_for(attached)
        if mode == "cancel":
            debugger.stdin.write(b"detach\n"); debugger.stdin.flush()
            close_gdb(debugger, reader, receipt)
            drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
            worker.close()
            if worker.tracer() != 0: raise ValueError("Debugger remained attached after cancellation")
            (controls / "release").write_text("normal")
        else:
            (controls / "release").write_text("fault" if mode == "fault" else "normal")
            def exited():
                drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
                return debugger.poll() is not None
            wait_for(exited, 15)
            close_gdb(debugger, reader, receipt)
            drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
        status = child.wait(timeout=5)
        if status != (1 if mode == "fault" else 0):
            raise ValueError("Live control original runner status changed")
        statuses = [packet["status"] for packet in receipt.get("gdbPackets", [])]
        if ("SIGSEGV" in statuses) != (mode == "fault"):
            raise ValueError("Live control fault attribution differs")
        cleanup = [packet for packet in receipt.get("gdbPackets", []) if packet["status"] == "debugger-cleanup"]
        if len(cleanup) != 1 or not cleanup[0]["debuggerReaped"] or cleanup[0].get("gap") or cleanup[0].get("forcedDebuggerExit"):
            raise ValueError("Live control debugger was not actually reaped")
        if mode == "fault" and receipt.get("faultProviderCustodyGap"):
            raise ValueError("Live fault provider bodies changed or were not bound before capture")
        receipt["originalReturnCode"] = status
        receipt["debuggerActuallyReaped"] = True
    finally:
        close_gdb(debugger, reader, receipt)
        if worker: worker.close()
        if child.poll() is None: child.kill(); child.wait()
        write_json(stage / "public/live-control.json", receipt)
        for public in (stage / "public").glob("*.json"):
            shutil.copyfile(public, output / "public" / ("live-" + mode + "-" + public.name))


def main():
    current = Path.cwd().resolve()
    controls = output / "live-controls"
    controls.mkdir(mode=0o700)
    summary = {"controls": [], "status": "unavailable"}
    try:
        with tempfile.TemporaryDirectory(prefix="owned-gdb-control-", dir=controls) as temporary:
            root = Path(temporary).resolve()
            package = root / PACKAGE
            (package / "src").mkdir(parents=True)
            (package / "package.json").write_text(json.dumps({"name": "owned-worker-control", "type": "module", "scripts": {"test": RECIPE}}))
            (package / "tsconfig.json").write_text((current / PACKAGE / "tsconfig.json").read_text())
            (root / "package.json").write_text('{"private":true,"type":"module"}')
            (root / "node_modules").symlink_to((current / "node_modules").resolve())
            (package / "node_modules").symlink_to((current / PACKAGE / "node_modules").resolve())
            checked(["git", "init", "-q"], root)
            checked(["git", "add", "package.json", PACKAGE + "/package.json", PACKAGE + "/tsconfig.json"], root)
            checked(["git", "-c", "user.name=Law", "-c", "user.email=law@example.invalid", "commit", "-qm", "test: controlled source"], root)
            for mode in ["normal", "cancel", "fault"]:
                stage = controls / mode; stage.mkdir(mode=0o700)
                for name in FILES: shutil.copyfile(output / name, stage / name)
                run_case(root, stage, stage, mode)
                summary["controls"].append(mode)
        summary["status"] = "passed"
        return 0
    except Exception as error:
        summary["gap"] = type(error).__name__
        return 1
    finally:
        write_json(output / "public/live-controls-summary.json", summary)


if __name__ == "__main__":
    sys.exit(main())
