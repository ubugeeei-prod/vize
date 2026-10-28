/** Adapt only synthetic perf framing; genuine decoder bytes remain unchanged. */
export function sentinelAdapterScript(decoder: string) {
  const body = String.raw`import queue, re, subprocess, sys, threading
MARKER = b"0xdeadbeefbadc0ffe"
command = [DECODER, *sys.argv[1:]]
if "--version" in sys.argv[1:]:
    raise SystemExit(subprocess.call(command))
child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
pending, failures = queue.Queue(), []
def forward():
    try:
        for line in sys.stdin.buffer:
            synthetic = line == b",\n"
            value = MARKER if synthetic else line.strip()
            if re.fullmatch(rb"(?:0x)?[0-9a-fA-F]+", value) is None:
                raise ValueError("non-address decoder request")
            address = int(value, 16)
            pending.put((address, synthetic))
            child.stdin.write(value + b"\n")
            child.stdin.flush()
    except (OSError, ValueError) as error:
        failures.append(str(error))
    finally:
        try:
            child.stdin.close()
        except OSError as error:
            failures.append(str(error))
thread = threading.Thread(target=forward, daemon=True)
thread.start()
active, body_lines = None, 0
def complete():
    if active is not None and (body_lines < 2 or body_lines % 2):
        raise ValueError("incomplete function/source frame")
try:
    for line in iter(child.stdout.readline, b""):
        if not line.endswith(b"\n"):
            raise ValueError("truncated decoder line")
        header = line.rstrip(b"\r\n")
        if re.fullmatch(rb"0x[0-9a-fA-F]+", header):
            complete()
            try:
                active = pending.get_nowait()
            except queue.Empty:
                raise ValueError("unsolicited address header") from None
            if int(header, 16) != active[0]:
                raise ValueError("address header differs from ordered request")
            body_lines = 0
            # perf waits for this delimiter before issuing its next request.
            # The synthetic address can own arbitrary tombstone debug frames.
            sys.stdout.buffer.write(b",\n" if active[1] else line)
        else:
            if active is None:
                raise ValueError("body without an address header")
            body_lines += 1
            if not active[1]:
                sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()
    complete()
    thread.join(timeout=1)
    if thread.is_alive() or not pending.empty():
        raise ValueError("missing decoder response or unfinished input")
except (OSError, ValueError) as error:
    failures.append(str(error))
    child.kill()
status = child.wait()
if failures:
    sys.stderr.write("addr2line sentinel frame failed: " + "; ".join(failures) + "\n")
raise SystemExit(1 if failures else status)
`;
  return "#!/usr/bin/env python3\n" + body.replace("DECODER", JSON.stringify(decoder));
}
