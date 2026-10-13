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


def cleanup_case(child, debugger, reader, packets, worker, leader, profile, root, stage, controls, receipt):
    close_gdb(debugger, reader, receipt)
    drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
    reaped = [packet for packet in receipt.get("gdbPackets", []) if packet["status"] == "debugger-cleanup"]
    safe_release = debugger is None or (len(reaped) == 1 and reaped[0].get("debuggerReaped") is True)
    if worker:
        worker.close()
        receipt["workerCleanup"] = list(worker.events)
        try:
            if "ESRCH-observation-gap" in worker.events:
                receipt["cleanupWorkerGone"] = "pidfd-ESRCH-observation-gap"
                safe_release = False
                raise ProcessLookupError()
            worker.check()
            receipt["cleanupWorkerBirthVerified"] = True
            receipt["cleanupTracerPid"] = worker.tracer()
            if receipt["cleanupTracerPid"] != 0:
                receipt["liveCleanupGap"] = "WorkerStillTraced"
                safe_release = False
            if "owned-worker-continued" not in worker.events: safe_release = False
        except ProcessLookupError:
            receipt["cleanupObservationGap"] = "ESRCH-observation-gap"
            safe_release = False
        except FileNotFoundError:
            receipt["cleanupObservationGap"] = "ProcFileMissing"
            safe_release = False
        except (OSError, ValueError) as error:
            receipt["liveCleanupGap"] = "WorkerCleanup" + type(error).__name__
            safe_release = False
    if child.poll() is None:
        # Release only the authored fixture input, after debugger cleanup and
        # verified pidfd resume; no descendant PID signalling or tree kill.
        if safe_release: (controls / "release").write_text("normal")
        else: receipt["liveCleanupGap"] = receipt.get("liveCleanupGap", "DebuggerReapingUnproved")
        try:
            receipt["cleanupOriginalReturnCode"] = child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            receipt["liveCleanupGap"] = "ToyRunnerDidNotDrain"
            child.kill()  # Only our own still-unreaped authored runner.
            try: receipt["cleanupOriginalReturnCode"] = child.wait(timeout=5)
            except subprocess.TimeoutExpired: receipt["liveCleanupGap"] = "ToyRunnerUnreaped"


def validate_case(receipt, mode, status):
    if status != (1 if mode in ["fault", "original-interrupt"] else 0):
        raise ValueError("Live control original runner status changed")
    packets = receipt.get("gdbPackets", [])
    statuses = [packet["status"] for packet in packets]
    if statuses.count("attached") != 1 or statuses.count("SIGSEGV") != (1 if mode == "fault" else 0):
        raise ValueError("Live control actor or fault attribution differs")
    if mode != "cancel" and statuses.count("exited") != 1:
        raise ValueError("Actual owned inferior exit event missing or duplicated")
    providers = {phase: [packet for packet in packets if packet.get('phase') == phase] for phase in ['debugger-provider-before', 'debugger-provider-after']}
    if any(len(values) != 1 for values in providers.values()): raise ValueError('Actual debugger provider identity missing')
    selected = lambda value: {key: value[key] for key in ['executable', 'elf', 'version']}
    if selected(providers['debugger-provider-before'][0]) != selected(providers['debugger-provider-after'][0]): raise ValueError('Actual debugger provider changed')
    callbacks = [packet for packet in packets if packet.get('phase') == 'callback-observations']
    if len(callbacks) != 1 or callbacks[0].get('ownedReady') is not True or callbacks[0].get('exceeded') or callbacks[0].get('afterAttach', 0) < 1: raise ValueError('Actual owned commanded continuation missing')
    allowed = {"UnexpectedWorkerStop", "InterruptOriginObserved", "DiagnosticCleanupInterrupt"} if mode in ["cancel", "original-interrupt"] else set()
    for packet in packets:
        if packet["status"] == "unavailable" and (packet.get("gap") not in allowed or
                (packet.get("gap") == "UnexpectedWorkerStop" and packet.get("signal") != "SIGINT")):
            raise ValueError("Unexpected or incomplete debugger stop")
    cleanup = [packet for packet in packets if packet["status"] == "debugger-cleanup"]
    if len(cleanup) != 1 or not cleanup[0]["debuggerReaped"] or cleanup[0].get("gap") or cleanup[0].get("forcedDebuggerExit") or cleanup[0].get("debuggerReturnCode") != 0:
        raise ValueError("Live control debugger was not actually reaped cleanly")
    if any(receipt.get(key) for key in ["markerGap", "readerGap", "gdbGap", "faultProviderGap", "faultProviderCustodyGap", "acknowledgementGap"]):
        raise ValueError("Live control source, provider or debugger observation incomplete")
    if mode == "cancel" and (receipt.get("cancellationTracerPid") != 0 or not cleanup[0].get("workerResumed") or
            not any(packet.get("diagnosticInterruptOriginMatch") is True for packet in packets) or
            not any(packet.get("phase") == "diagnostic-signal-discarded" for packet in packets) or "detached" not in statuses):
        raise ValueError("Actual cancellation detach, origin, discard or resume proof missing")
    if mode == "original-interrupt" and not any(packet.get("diagnosticInterruptOriginMatch") is False for packet in packets):
        raise ValueError("Genuine worker SIGINT was not distinguished during cancellation")
    if mode in ['fault', 'original-interrupt']:
        onward = [packet for packet in packets if packet.get('phase') == 'original-signal-onward-policy']
        expected = 'SIGSEGV' if mode == 'fault' else 'SIGINT'
        if len(onward) != 1 or onward[0].get('signal') != expected or onward[0].get('stop') is not False or onward[0].get('pass') is not True:
            raise ValueError('Actual original signal onward delivery policy missing')


def qualify_status(receipt, mode, status):
    receipt['originalReturnCode'] = status  # Retain measured status before any refusal.
    validate_case(receipt, mode, status)


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
watcher.close();const mode=readFileSync(directory+'/release','utf8');if(mode==='fault')process.kill(process.pid,'SIGSEGV');if(mode==='interrupt')process.kill(process.pid,'SIGINT');});
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
        def running():
            drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
            return any(packet.get("phase") == "inferior-running" and packet.get("pid") == worker.identity["pid"] for packet in receipt.get("gdbPackets", []))
        wait_for(running)
        if mode == "cancel":
            debugger.stdin.write(b"detach\n"); debugger.stdin.flush()
            close_gdb(debugger, reader, receipt)
            drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
            worker.close()
            worker.check()
            receipt["cancellationTracerPid"] = worker.tracer()
            if receipt["cancellationTracerPid"] != 0: raise ValueError("Debugger remained attached after cancellation")
            (controls / "release").write_text("normal")
        else:
            if mode == "original-interrupt": (stage / "gdb-private/detach-requested").write_text("")
            (controls / "release").write_text("interrupt" if mode == "original-interrupt" else "fault" if mode == "fault" else "normal")
            def exited():
                drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
                return debugger.poll() is not None
            wait_for(exited, 15)
            close_gdb(debugger, reader, receipt)
            drain_packets(packets, debugger, worker, leader, profile, root, stage, receipt)
        status = child.wait(timeout=5)
        qualify_status(receipt, mode, status)
        receipt["debuggerActuallyReaped"] = True
    except Exception as error:
        receipt["liveControlGap"] = type(error).__name__
        raise
    finally:
        cleanup_case(child, debugger, reader, packets, worker, locals().get("leader"), profile, root, stage, controls, receipt)
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
            for mode in ["normal", "cancel", "fault", "original-interrupt"]:
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
