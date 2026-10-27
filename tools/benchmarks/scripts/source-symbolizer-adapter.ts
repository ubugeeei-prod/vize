/** Adapt only perf's framing marker; genuine decoder bytes remain unchanged. */
export function sentinelAdapterScript(decoder: string) {
  const body = String.raw`import subprocess, sys, threading
MARKER = b"0xdeadbeefbadc0ffe"
child = subprocess.Popen([DECODER, *sys.argv[1:]], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
failures = []
def forward():
    try:
        for line in sys.stdin.buffer:
            child.stdin.write(MARKER + b"\n" if line == b",\n" else line)
            child.stdin.flush()
    except OSError as error:
        failures.append(str(error))
    finally:
        try:
            child.stdin.close()
        except OSError as error:
            failures.append(str(error))
thread = threading.Thread(target=forward, daemon=True)
thread.start()
while True:
    line = child.stdout.readline()
    if not line:
        break
    if line.rstrip(b"\r\n").lower() == MARKER:
        function = child.stdout.readline()
        source = child.stdout.readline()
        if function != b"??\n" or source != b"??:0\n":
            failures.append("reserved marker did not return the exact unknown frame")
            child.kill()
            break
        sys.stdout.buffer.write(b",\n")
    else:
        sys.stdout.buffer.write(line)
    sys.stdout.buffer.flush()
status = child.wait()
if failures:
    sys.stderr.write("addr2line sentinel frame failed: " + "; ".join(failures) + "\n")
raise SystemExit(1 if failures else status)
`;
  return "#!/usr/bin/env python3\n" + body.replace("DECODER", JSON.stringify(decoder));
}
