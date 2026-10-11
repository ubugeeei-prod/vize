"""GDB-only script: one qualified inferior, bounded diagnostics, no arguments."""
import json
import os
from pathlib import Path
import signal
import sys
sys.dont_write_bytecode = True
import time

import gdb

sys.path.insert(0, str(Path(__file__).parent))
from owned_worker_identity import HASH_METRICS, same_birth, select, state

LIMIT = 131072
BEGIN = "VIZE_OWNED_WORKER_BEGIN"
END = "VIZE_OWNED_WORKER_END"


def publish(value):
    value["hashing"] = dict(HASH_METRICS)
    body = json.dumps(value, separators=(",", ":"))
    if len(body.encode()) > LIMIT:
        value = {"status": "unavailable", "gap": "DiagnosticTextExceedsBound", "hashing": dict(HASH_METRICS)}
        body = json.dumps(value)
    gdb.write(BEGIN + "\n" + body + "\n" + END + "\n")
    gdb.flush()


def verified(config):
    actor = config["worker"]["actor"]
    signal.pidfd_send_signal(config["pidfd"], 0)
    if not same_birth(state(actor["pid"]), actor):
        raise ValueError("Worker birth or executable changed")
    match = select(actor["pid"], config["leader"], config["profile"],
                   Path(config["root"]), owner_uid=actor["uid"])
    if match is None or match["actor"] != actor:
        # State may transition from stopped to tracing stop while retaining birth.
        if match is None or not same_birth(match["actor"], actor):
            raise ValueError("Private isolated worker profile changed")
    return match


def fault_text():
    texts = []
    original_thread = gdb.selected_thread()
    if original_thread is None:
        raise ValueError("Faulting thread identity unavailable")
    faulting_thread = {"number": original_thread.num, "ptid": list(original_thread.ptid)}
    registers = gdb.execute("info registers", to_string=True)
    pc = hex(int(gdb.parse_and_eval("$pc")))
    threads = gdb.selected_inferior().threads()
    if len(threads) > 64:
        raise ValueError("Worker thread count exceeds bound")
    try:
        for thread in threads:
            thread.switch()
            texts.append({"thread": thread.num, "backtrace": gdb.execute("bt 32", to_string=True)})
    finally:
        original_thread.switch()
    return {"faultingThread": faulting_thread, "pc": pc, "threads": texts,
            "registers": registers, "libraries": gdb.execute("info sharedlibrary", to_string=True)}


def cleanup_interrupt(pending_signal):
    if pending_signal != "SIGINT" or not Path(__file__).with_name("detach-requested").exists():
        return False
    try:
        number = int(gdb.parse_and_eval("$_siginfo.si_signo"))
        code = int(gdb.parse_and_eval("$_siginfo.si_code"))
        sender = int(gdb.parse_and_eval("$_siginfo._sifields._kill.si_pid"))
        uid = int(gdb.parse_and_eval("$_siginfo._sifields._kill.si_uid"))
        # Linux SI_USER/SI_TKILL from this actual GDB process, never a caller PID.
        match = number == signal.SIGINT and code in [0, -6] and sender == os.getpid() and uid == os.getuid()
        publish({"status": "unavailable", "gap": "InterruptOriginObserved",
                 "diagnosticInterruptOriginMatch": match, "signalCode": code,
                 "senderPid": sender, "senderUid": uid, "actualDebuggerPid": os.getpid()})
        return match
    except Exception as error:
        publish({"status": "unavailable", "gap": "InterruptOrigin" + type(error).__name__})
        return False


def run(config):
    attached = False
    pending_signal = None
    exited = []
    def stopped(event):
        nonlocal pending_signal
        pending_signal = event.stop_signal if isinstance(event, gdb.SignalEvent) else "UnexpectedStop"
    def exit_event(event):
        exited.append(getattr(event, "exit_code", None))
    gdb.events.stop.connect(stopped)
    gdb.events.exited.connect(exit_event)
    owned_fd = None
    try:
        if not same_birth(state(config["worker"]["actor"]["pid"]), config["worker"]["actor"]):
            raise ValueError("Worker birth changed before opening GDB pidfd")
        owned_fd = os.pidfd_open(config["worker"]["actor"]["pid"])
        config["pidfd"] = owned_fd
        verified(config)
        attached = True  # Even a partially failed attach must attempt detach.
        attach_started = time.monotonic_ns()
        gdb.execute("attach " + str(config["worker"]["actor"]["pid"]), to_string=True)
        attach_finished = time.monotonic_ns()
        verified(config)
        if gdb.selected_inferior().pid != config["worker"]["actor"]["pid"]:
            raise ValueError("GDB selected a different inferior")
        publish({"status": "attached", "pid": config["worker"]["actor"]["pid"],
                 "startTicks": config["worker"]["actor"]["startTicks"],
                 "numericAttachIntervalNs": [attach_started, attach_finished],
                 "numericAttachAtomic": False})
        signal.pidfd_send_signal(config["pidfd"], signal.SIGCONT)
        pending_signal = None
        gdb.execute("continue", to_string=True)
        if exited:
            attached = False
            publish({"status": "exited", "exitCode": exited[-1]})
            return
        verified(config)
        if pending_signal != "SIGSEGV":
            publish({"status": "unavailable", "gap": "UnexpectedWorkerStop", "signal": pending_signal})
            return
        publish({"status": "SIGSEGV", **fault_text()})
        # The main helper takes physical maps/provider hashes while this worker
        # remains stopped; acknowledgement never contains guest content.
        acknowledgement = sys.stdin.readline(32)
        if acknowledgement != "captured\n":
            publish({"status": "unavailable", "gap": "MissingProviderAcknowledgement"})
        verified(config)
        pending_signal = None
        gdb.execute("continue", to_string=True)  # handle SIGSEGV pass preserves it.
        if exited:
            attached = False
            publish({"status": "exited", "exitCode": exited[-1]})
        else:
            publish({"status": "unavailable", "gap": "PostFaultStop"})
    except Exception as error:
        publish({"status": "unavailable", "gap": type(error).__name__})
    finally:
        if attached:
            diagnostic_interrupt = cleanup_interrupt(pending_signal)
            if diagnostic_interrupt:
                publish({"status": "unavailable", "gap": "DiagnosticCleanupInterrupt"})
                try:
                    gdb.execute("queue-signal 0", to_string=True)
                except Exception as error:
                    publish({"status": "unavailable", "gap": "DiscardDiagnosticSignal" + type(error).__name__})
            if pending_signal and not diagnostic_interrupt and pending_signal.startswith("SIG") and hasattr(signal, pending_signal):
                try:
                    # A diagnostic exception must not consume the original fault.
                    original_signal = pending_signal
                    pending_signal = None
                    gdb.execute("signal " + original_signal, to_string=True)
                except Exception as error:
                    publish({"status": "unavailable", "gap": "PassSignal" + type(error).__name__})
            if not exited:
                try:
                # Detach the currently held inferior; no PID lookup or tree walk.
                    gdb.execute("detach", to_string=True)
                    publish({"status": "detached"})
                except Exception as error:
                    publish({"status": "unavailable", "gap": "Detach" + type(error).__name__})
        if owned_fd is not None:
            os.close(owned_fd)
        gdb.events.stop.disconnect(stopped)
        gdb.events.exited.disconnect(exit_event)


if __name__ == "__main__":
    publish({"status": "debugger-phase", "phase": "script-started"})
    for setting in ["set pagination off", "set confirm off", "set auto-load off",
                    "set debuginfod enabled off", "set print frame-arguments none",
                    "set print entry-values no", "set follow-fork-mode parent",
                    "set detach-on-fork on", "handle SIGSEGV stop noprint pass",
                    "handle SIGSTOP nostop noprint nopass"]:
        gdb.execute(setting, to_string=True)
    run(json.loads(Path(__file__).with_name("attach-private.json").read_text()))
