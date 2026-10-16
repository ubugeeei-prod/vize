"""Observe the explicit worker diagnostic without changing runner execution."""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
sys.dont_write_bytecode = True
import time

from owned_worker_capture import Worker, authenticate, close_gdb, git_identity, start_gdb, write_json
from owned_worker_identity import HASH_METRICS, prepare_profile, providers, select, state


def source_census(output, phase, receipt):
    # Reuse only the merged read-only whole census; never its command/kernel path.
    command = """import { pathToFileURL } from 'node:url';
const module = await import(pathToFileURL(process.argv[1]).href);
const packet = module.sourceSnapshot(true);
process.stdout.write(JSON.stringify({packet, hashing: module.hashMeasurements}));"""
    observed = json.loads(subprocess.check_output([
        "node", "--input-type=module", "-e", command, str(output / "native-fault-support.ts")
    ], text=True))
    packet = observed["packet"]
    write_json(output / ("public/source-" + phase + ".json"), packet)
    unavailable = [item["file"] for item in packet["files"] if "unavailable" in item]
    receipt["sourceCensus" + phase.title()] = {"files": len(packet["files"]), "unavailable": unavailable,
                                           "hashing": observed["hashing"],
                                           "gitAuthority": "--no-replace-objects for every Git call"}
    if unavailable:
        raise ValueError("Whole physical source census has observation gaps")
    if phase == "after":
        before = json.loads((output / "public/source-before.json").read_text())
        bodies_before = {item["file"]: item for item in before["files"]}
        bodies_after = {item["file"]: item for item in packet["files"]}
        receipt["sourceDrift"] = [name for name in sorted(bodies_before.keys() | bodies_after.keys()) if bodies_before.get(name) != bodies_after.get(name)]
        receipt["sourceCensusAfter"]["wholePhysicalBodiesEqual"] = not receipt["sourceDrift"]
        if before["git"] != packet["git"] or before["diffSha256"] != packet["diffSha256"]:
            receipt["sourceCustodyGap"] = "GitIdentityOrPhysicalDiffChanged"


def physical_packet(worker, leader, profile, root):
    worker.check()
    selected = select(worker.identity["pid"], leader, profile, root)
    if selected is None:
        raise ValueError("Worker profile is no longer exact")
    packet = {**selected, **providers(worker.identity["pid"])}
    worker.check()
    return packet


def drain_packets(packets, debugger, worker, leader, profile, root, output, receipt):
    while packets is not None and not packets.empty():
        packet = packets.get_nowait()
        receipt.setdefault("gdbPackets", []).append(packet)
        if packet["status"] == "unavailable":
            receipt["debuggerGap"] = packet.get("gap", "UnspecifiedDebuggerGap")
        if packet["status"] == "debugger-cleanup" and (packet.get("gap") or packet.get("forcedDebuggerExit") or not packet.get("debuggerReaped")):
            receipt["debuggerGap"] = packet.get("gap", "ForcedOrUnreapedDebuggerExit")
        if packet["status"] == "SIGSEGV":
            try:
                fault = physical_packet(worker, leader, profile, root)
                write_json(output / "public/worker-fault.json", fault)
                before = json.loads((output / "public/worker-before.json").read_text())
                earlier = {item["path"]: item for item in before["providers"]}
                later = {item["path"]: item for item in fault["providers"]}
                changed = [name for name in sorted(earlier.keys() | later.keys()) if earlier.get(name) != later.get(name)]
                if changed:
                    receipt["faultProviderCustodyGap"] = changed
            except Exception as error:
                receipt["faultProviderGap"] = type(error).__name__
            finally:
                if debugger is not None and debugger.poll() is None:
                    try:
                        debugger.stdin.write(b"captured\n")
                        debugger.stdin.flush()
                    except OSError as error:
                        receipt["acknowledgementGap"] = type(error).__name__


def live_controls(output, environment, receipt):
    if not shutil.which("gdb"):
        result = subprocess.run(["sudo", "-n", "apt-get", "install", "-y", "--no-install-recommends", "gdb"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=120)
        if result.returncode != 0:
            raise ValueError("GDB unavailable")
    controls = subprocess.run([sys.executable, str(output / "owned_worker_live_laws.py"), str(output)], env=environment, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=120)
    if controls.returncode != 0:
        raise ValueError("Actual debugger lifecycle controls failed")
    receipt["liveControls"] = "original status/SIGINT, fault pass and cancellation/reaping passed"


def controls_only(output):
    receipt = {"diagnostic": "authored Linux worker controls only", "originalRunnerStarted": False,
               "qualification": "none; authored diagnostic controls"}
    environment = dict(os.environ)
    for key in ["VIZE_OWNED_WORKER_CAPTURE", "VIZE_OWNED_WORKER_DIRECTORY"]:
        environment.pop(key, None)
    status = 1
    try:
        authenticate(output)
        source_census(output, "before", receipt)
        live_controls(output, environment, receipt)
        status = 0
    except Exception as error:
        receipt["controlsGap"] = type(error).__name__
    finally:
        try:
            source_census(output, "after", receipt)
            if receipt.get("sourceDrift") or receipt.get("sourceCustodyGap"): status = 1
        except Exception as error:
            receipt["sourceAfterGap"] = type(error).__name__; status = 1
        receipt["observerHashing"] = dict(HASH_METRICS)
        receipt["controlReturnCode"] = status
        write_json(output / "public/controls-only-outcome.json", receipt)
    return status


def observe(command, output):
    root, receipt = Path.cwd().resolve(), {"diagnostic": "explicit-owned-worker-ptrace", "cores": "unavailable; no kernel changes", "originalRunner": "argv/env/stdin/stdout/stderr unchanged, once", "qualification": "none; changed worker scheduling"}
    child, worker, debugger, reader, profile, leader = None, None, None, None, None, None
    packets, prior_handlers, attempted = None, {}, False
    original_environment = dict(os.environ)
    for key in ["VIZE_OWNED_WORKER_CAPTURE", "VIZE_OWNED_WORKER_DIRECTORY"]:
        original_environment.pop(key, None)
    try:
        try:
            authenticate(output)
            source_census(output, "before", receipt)
            live_controls(output, original_environment, receipt)
            probe = output / "worker-profile.test.mjs"
            shutil.copyfile(output / "owned_worker_probe.mjs", probe)
            profile = prepare_profile(root, probe, output / "probe-private.json", original_environment)
            public_profile = {key: value for key, value in profile.items() if key != "privateArgv"}
            write_json(output / "public/profile.json", public_profile)
        except Exception as error:
            receipt["setupGap"] = type(error).__name__
            profile = None
        receipt["runnerStartedAt"] = time.time()
        child = subprocess.Popen(command, env=original_environment)
        try:
            leader = state(child.pid) if profile else None
        except Exception as error:
            receipt["leaderGap"] = type(error).__name__
            profile = None
        def forward(signum, frame):
            receipt["forwardedSignal"] = signum
            if child.poll() is None:
                child.send_signal(signum)
            if debugger is not None and debugger.poll() is None:
                debugger.send_signal(signal.SIGINT)
        for signum in [signal.SIGINT, signal.SIGTERM]:
            prior_handlers[signum] = signal.signal(signum, forward)
        while child.poll() is None:
            if profile and not attempted:
                for directory in Path("/proc").iterdir():
                    if not directory.name.isdigit():
                        continue
                    try:
                        selected = select(int(directory.name), leader, profile, root)
                        if selected is None:
                            continue
                        attempted = True
                        worker = Worker(selected["actor"])
                        worker.stop()
                        selected = select(selected["actor"]["pid"], leader, profile, root)
                        if selected is None:
                            raise ValueError("Stopped worker profile differs")
                        write_json(output / "public/worker-before.json", physical_packet(worker, leader, profile, root))
                        debugger, packets, reader = start_gdb(output, selected, leader, profile, root, receipt)
                        receipt["debuggerStartedAt"] = time.time()
                        break
                    except Exception as error:
                        if attempted:
                            receipt["attachGap"] = type(error).__name__
                            if worker:
                                worker.close()
                            break
            if debugger is not None:
                drain_packets(packets, debugger, worker, leader, profile, root, output, receipt)
                if debugger.poll() is not None or time.time() - receipt["debuggerStartedAt"] > 90:
                    close_gdb(debugger, reader, receipt)
                    drain_packets(packets, debugger, worker, leader, profile, root, output, receipt)
                    worker.close()
                    debugger = None
            time.sleep(.02)
        receipt["originalReturnCode"] = child.returncode
    except Exception as error:
        receipt["observerGap"] = type(error).__name__
        close_gdb(debugger, reader, receipt)
        drain_packets(packets, debugger, worker, leader, profile, root, output, receipt)
        debugger = None
        if worker:
            worker.close()
        if child is not None:
            receipt["originalReturnCode"] = child.wait()
    finally:
        close_gdb(debugger, reader, receipt)
        drain_packets(packets, debugger, worker, leader, profile, root, output, receipt)
        if worker:
            worker.close()
            receipt["workerCleanup"] = worker.events
        for signum, handler in prior_handlers.items():
            signal.signal(signum, handler)
        receipt["runnerFinishedAt"] = time.time()
        try:
            receipt["sourceAfter"] = git_identity()
            source_census(output, "after", receipt)
        except Exception as error:
            receipt["sourceAfterGap"] = type(error).__name__
        statuses = [item["status"] for item in receipt.get("gdbPackets", [])]
        if "SIGSEGV" in statuses:
            gaps = any("Gap" in key for key in receipt) or receipt.get("markerGap") or receipt.get("sourceDrift")
            receipt["captureStatus"] = "partial SIGSEGV diagnostic; custody gaps" if gaps else "authenticated SIGSEGV diagnostic; no qualification credit"
        elif "attached" in statuses and "exited" in statuses:
            receipt["captureStatus"] = "owned worker exited without observed SIGSEGV; original failure remains"
        else:
            receipt["captureStatus"] = "unavailable; no authenticated fault markers"
        try:
            receipt["observerHashing"] = dict(HASH_METRICS)
            write_json(output / "public/outcome.json", receipt)
        except OSError:
            pass  # Evidence upload fails separately; original status stays intact.
    return child.returncode
