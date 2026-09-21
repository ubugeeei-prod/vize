#!/usr/bin/env node
// TS-45 transcript tap: a byte-transparent proxy between an editor and the
// real `vize lsp`. Every framed JSON-RPC message is appended to the JSONL file
// named by VIZE_CONFORMANCE_TRANSCRIPT *before* it is forwarded, so the
// transcript order is the causal order each side observed. The conformance
// judge (`conformance.ts`) reads only this transcript; no editor reports its
// own verdict.
//
// usage: lsp-tap.mjs <server> [server-args...]
import { spawn } from "node:child_process";
import fs from "node:fs";

const [server, ...serverArgs] = process.argv.slice(2);
const transcriptPath = process.env.VIZE_CONFORMANCE_TRANSCRIPT;
if (!server || !transcriptPath) {
  process.stderr.write("usage: VIZE_CONFORMANCE_TRANSCRIPT=<file> lsp-tap.mjs <server> [args]\n");
  process.exit(2);
}

const transcript = fs.openSync(transcriptPath, "a");
const session = process.pid;
const startedAt = Date.now();

function record(entry) {
  const line = JSON.stringify({ session, t: Date.now() - startedAt, ...entry });
  fs.writeSync(transcript, `${line}\n`);
}

/** Incremental `Content-Length` framing; returns every complete message body. */
function frameReader(direction) {
  let buffer = Buffer.alloc(0);
  return (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const headerEnd = buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) return;
      const header = buffer.subarray(0, headerEnd).toString("ascii");
      const length = /content-length:\s*(\d+)/iu.exec(header);
      if (!length) {
        record({ dir: direction, error: `frame without Content-Length: ${header}` });
        buffer = buffer.subarray(headerEnd + 4);
        continue;
      }
      const bodyStart = headerEnd + 4;
      const bodyEnd = bodyStart + Number(length[1]);
      if (buffer.length < bodyEnd) return;
      const body = buffer.subarray(bodyStart, bodyEnd).toString("utf8");
      buffer = buffer.subarray(bodyEnd);
      try {
        record({ dir: direction, msg: JSON.parse(body) });
      } catch (error) {
        record({ dir: direction, error: `invalid JSON: ${error.message}`, body });
      }
    }
  };
}

// The server's exit is recorded by a `/bin/sh` wrapper, not by this process:
// editors may SIGKILL their language-server child right after sending `exit`
// (Helix drops it with `kill_on_drop`), which would lose an in-process record.
// The wrapper outlives the tap, sees the server's real status, and appends
// `{"event":"exit","code":…}` for this session. stderr is inherited, not
// piped: the server logs into the editor's own stderr handle as it would
// without the tap.
const exitRecorder = [
  'code=0; "$0" "$@" || code=$?',
  `printf '{"session":%s,"t":null,"event":"exit","code":%s}\\n' "$VIZE_TAP_SESSION" "$code" >> "$VIZE_CONFORMANCE_TRANSCRIPT"`,
  'exit "$code"',
].join("; ");
const child = spawn("/bin/sh", ["-c", exitRecorder, server, ...serverArgs], {
  env: { ...process.env, VIZE_TAP_SESSION: String(session) },
  stdio: ["pipe", "pipe", "inherit"],
});
record({ event: "spawn", argv: [server, ...serverArgs], cwd: process.cwd() });

const clientToServer = frameReader("c2s");
const serverToClient = frameReader("s2c");

process.stdin.on("data", (chunk) => {
  clientToServer(chunk);
  child.stdin.write(chunk);
});
process.stdin.on("end", () => child.stdin.end());
child.stdin.on("error", () => {});
child.stdout.on("data", (chunk) => {
  serverToClient(chunk);
  process.stdout.write(chunk);
});
process.stdout.on("error", () => {});

for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  process.on(signal, () => {
    record({ event: "signal", signal });
    child.kill(signal);
  });
}

child.on("error", (error) => {
  record({ event: "spawn-error", message: error.message });
  process.exit(127);
});
// `close` (not `exit`) fires after the server's stdout drained through the tap.
child.on("close", (code) => {
  fs.closeSync(transcript);
  process.exitCode = code ?? 1;
  // Let the last server frame drain to the editor before exiting.
  process.stdout.end(() => process.exit(code ?? 1));
});
