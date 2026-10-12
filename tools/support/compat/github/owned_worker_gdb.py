"""GDB-only script: one qualified inferior, bounded diagnostics, no arguments."""
import hashlib
import json
import os
from pathlib import Path
import signal
import re
import sys
sys.dont_write_bytecode = True
import time

import gdb

sys.path.insert(0, str(Path(__file__).parent))
from owned_worker_identity import HASH_METRICS, elf_identity, file_identity, same_birth, select, state

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


def debugger_identity():
    path = Path('/proc/self/exe').resolve(strict=True)
    version = gdb.VERSION
    if not isinstance(version, str): raise ValueError('Debugger version unavailable')
    projected = {'bytes': len(version.encode()), 'sha256': hashlib.sha256(version.encode()).hexdigest()}
    if re.fullmatch(r'[0-9]+(?:\.[0-9]+){1,5}(?:[-+][0-9A-Za-z.~:+-]{1,48})?', version): projected['value'] = version
    return {'executable': file_identity(path), 'elf': elf_identity(path), 'version': projected}


def private_text(name, text):
    body = text.encode()
    selected = body[:LIMIT]
    with os.fdopen(os.open(Path(__file__).with_name(name), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), 'wb') as stream:
        stream.write(selected)
    return {'bytes': len(body), 'sha256': hashlib.sha256(body).hexdigest(), 'retainedBytes': len(selected),
            'retainedSha256': hashlib.sha256(selected).hexdigest(), 'truncated': len(body) > LIMIT, 'uploaded': False}


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


def verify_policy(name, flags):
    text = gdb.execute("info signals " + name, to_string=True)
    rows = [line.split() for line in text.splitlines() if line.split()[:1] == [name]]
    if len(text.encode()) > 8192 or len(rows) != 1 or rows[0][1:4] != flags:
        raise ValueError("Actual debugger signal stop/pass policy differs")


def onward_policy(name):
    if not re.fullmatch(r'SIG[A-Z0-9]{1,12}', name) or not hasattr(signal, name):
        raise ValueError('Original signal identity unavailable')
    # A runtime handler can re-raise the same signal. Capture one fault, then
    # preserve signal delivery without assuming its first delivery exits.
    gdb.execute("handle " + name + " nostop noprint pass", to_string=True)
    verify_policy(name, ["No", "No", "Yes"])
    publish({'status': 'debugger-phase', 'phase': 'original-signal-onward-policy',
             'signal': name, 'stop': False, 'pass': True})


def configure():
    for setting in ["set pagination off", "set confirm off", "set auto-load off",
                    "set debuginfod enabled off", "set print frame-arguments none",
                    "set print entry-values no", "set follow-fork-mode parent",
                    "set detach-on-fork on", "handle SIGSEGV stop print pass",
                    "handle SIGCONT nostop noprint pass",
                    "handle SIGSTOP nostop noprint nopass"]:
        gdb.execute(setting, to_string=True)
    for name, flags in [("SIGSEGV", ["Yes", "Yes", "Yes"]), ("SIGCONT", ["No", "No", "Yes"])]:
        verify_policy(name, flags)
    publish({"status": "debugger-phase", "phase": "signal-policy-verified",
             "sigsegvStop": True, "sigsegvPass": True, "sigcontStop": False, "sigcontPass": True})


def run(config):
    attached = False
    detaching = False
    inferior_number = None
    pending_signal = None
    continued_observed = False
    resume_requested = False
    callback_counts = {'duringAttach': 0, 'beforeCommand': 0, 'afterAttach': 0, 'ownedReady': False}
    debugger_before = None
    exited = []
    def owned_inferior():
        inferior = gdb.selected_inferior()
        if inferior_number is None or inferior.num != inferior_number or inferior.pid != config["worker"]["actor"]["pid"]:
            raise ValueError("Different owned inferior")
        return verified(config)
    def stopped(event):
        nonlocal pending_signal
        pending_signal = event.stop_signal if isinstance(event, gdb.SignalEvent) else "UnexpectedStop"
    def exit_event(event):
        if detaching: return  # Detach events do not prove the worker exited.
        if inferior_number is None or event.inferior.num != inferior_number:
            publish({"status": "unavailable", "gap": "ForeignExitedInferior"})
            return
        exited.append(getattr(event, "exit_code", None))
        if len(exited) == 1: publish({"status": "exited", "exitCode": exited[-1]})
    def continued(event):
        nonlocal continued_observed
        key = 'duringAttach' if inferior_number is None else 'afterAttach' if resume_requested else 'beforeCommand'
        if callback_counts[key] >= 64:
            callback_counts['exceeded'] = True
            return
        callback_counts[key] += 1
        if inferior_number is None or not resume_requested:
            return  # Linux attach can emit cont for newly discovered LWPs.
        if continued_observed: return  # Readiness is the first actual resume, not post-fault death.
        continued_observed = True
        try:
            owned_inferior()
        except Exception as error:
            publish({"status": "unavailable", "gap": "ContinuedActor" + type(error).__name__})
            return
        callback_counts['ownedReady'] = True
        publish({"status": "debugger-phase", "phase": "inferior-running",
                 "pid": config["worker"]["actor"]["pid"]})
    hooks = []
    owned_fd = None
    try:
        debugger_before = debugger_identity()
        publish({'status': 'debugger-phase', 'phase': 'debugger-provider-before', **debugger_before})
        configure()
        for registry, handler in [(gdb.events.stop, stopped), (gdb.events.exited, exit_event), (gdb.events.cont, continued)]:
            registry.connect(handler); hooks.append((registry, handler))
        if not same_birth(state(config["worker"]["actor"]["pid"]), config["worker"]["actor"]):
            raise ValueError("Worker birth changed before opening GDB pidfd")
        owned_fd = os.pidfd_open(config["worker"]["actor"]["pid"])
        config["pidfd"] = owned_fd
        verified(config)
        attached = True  # Even a partially failed attach must attempt detach.
        attach_started = time.monotonic_ns()
        text = gdb.execute("attach " + str(config["worker"]["actor"]["pid"]), to_string=True)
        attach_finished = time.monotonic_ns()
        projection = private_text('attach-command-output-private.txt', text)
        publish({'status': 'debugger-phase', 'phase': 'attach-output-authority', **projection})
        if projection['truncated']: raise ValueError('Attach command output exceeds bound')
        verified(config)
        if gdb.selected_inferior().pid != config["worker"]["actor"]["pid"]:
            raise ValueError("GDB selected a different inferior")
        inferior_number = gdb.selected_inferior().num
        publish({"status": "attached", "pid": config["worker"]["actor"]["pid"],
                 "startTicks": config["worker"]["actor"]["startTicks"],
                 "numericAttachIntervalNs": [attach_started, attach_finished],
                 "numericAttachAtomic": False})
        signal.pidfd_send_signal(config["pidfd"], signal.SIGCONT)
        pending_signal = None
        resume_requested = True
        gdb.execute("continue", to_string=False)  # Event markers bypass command-output capture.
        if exited:
            attached = False
            return
        owned_inferior()
        if pending_signal != "SIGSEGV":
            publish({"status": "unavailable", "gap": "UnexpectedWorkerStop", "signal": pending_signal})
            return
        publish({"status": "SIGSEGV", **fault_text()})
        # The main helper takes physical maps/provider hashes while this worker
        # remains stopped; acknowledgement never contains guest content.
        acknowledgement = sys.stdin.readline(32)
        if acknowledgement != "captured\n":
            publish({"status": "unavailable", "gap": "MissingProviderAcknowledgement"})
        owned_inferior()
        onward_policy("SIGSEGV")
        pending_signal = None
        gdb.execute("continue", to_string=False)  # handle SIGSEGV pass preserves it.
        if exited:
            attached = False
        else:
            observed = pending_signal if pending_signal and hasattr(signal, pending_signal) else 'UnexpectedStop'
            publish({"status": "unavailable", "gap": "PostFaultStop", "signal": observed})
    except Exception as error:
        publish({"status": "unavailable", "gap": type(error).__name__})
    finally:
        if attached:
            can_pass = False
            try: owned_inferior(); can_pass = True
            except Exception as error: publish({"status": "unavailable", "gap": "CleanupActor" + type(error).__name__})
            diagnostic_interrupt = cleanup_interrupt(pending_signal) if can_pass else False
            if diagnostic_interrupt:
                publish({"status": "unavailable", "gap": "DiagnosticCleanupInterrupt"})
                try:
                    gdb.execute("queue-signal 0", to_string=True)
                    publish({"status": "debugger-phase", "phase": "diagnostic-signal-discarded"})
                except Exception as error:
                    publish({"status": "unavailable", "gap": "DiscardDiagnosticSignal" + type(error).__name__})
            if can_pass and pending_signal and not diagnostic_interrupt and pending_signal.startswith("SIG") and hasattr(signal, pending_signal):
                try:
                    # A diagnostic exception must not consume the original fault.
                    original_signal = pending_signal
                    owned_inferior()
                    onward_policy(original_signal)
                    pending_signal = None
                    gdb.execute("signal " + original_signal, to_string=False)
                except Exception as error:
                    publish({"status": "unavailable", "gap": "PassSignal" + type(error).__name__})
            if not exited:
                try:
                # Detach the currently held inferior; no PID lookup or tree walk.
                    detaching = True
                    gdb.execute("detach", to_string=True)
                    publish({"status": "detached"})
                except Exception as error:
                    publish({"status": "unavailable", "gap": "Detach" + type(error).__name__})
        if owned_fd is not None:
            os.close(owned_fd)
        for registry, handler in reversed(hooks): registry.disconnect(handler)
        publish({'status': 'debugger-phase', 'phase': 'callback-observations', **callback_counts})
        if callback_counts.get('exceeded'): publish({'status': 'unavailable', 'gap': 'CallbackCountExceedsBound'})
        if debugger_before is not None:
            try:
                debugger_after = debugger_identity()
                publish({'status': 'debugger-phase', 'phase': 'debugger-provider-after', **debugger_after})
                if debugger_after != debugger_before: raise ValueError('Debugger provider changed')
            except Exception as error:
                publish({'status': 'unavailable', 'gap': 'DebuggerProvider' + type(error).__name__})


if __name__ == "__main__":
    publish({"status": "debugger-phase", "phase": "script-started"})
    run(json.loads(Path(__file__).with_name("attach-private.json").read_text()))
