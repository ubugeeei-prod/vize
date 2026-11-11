/**
 * Recording content-mapper proxy for content-mapper-replay-record.ts:
 *
 *   node content-mapper-replay-proxy.ts <stream.bin|-> <timeline.jsonl|-> -- <exec...>
 *
 * Forwards every byte both ways unchanged. It appends the host's bytes to
 * `stream.bin` (a replayable request stream) and one line per frame to
 * `timeline.jsonl`: `{t, dir, id, method}`, `t` in ms since the proxy
 * started, `dir` `req` (host → mapper) or `rep`. The proxy reads frames only
 * as far as their ids, so it holds no reply back. `%p` in either path is
 * the proxy's pid, for hosts that start more than one mapper.
 */

import { spawn } from "node:child_process";
import { openSync, writeSync } from "node:fs";

const separator = process.argv.indexOf("--");
const [streamPath, timelinePath] = process.argv.slice(2, separator);
const [command, ...args] = process.argv.slice(separator + 1);
if (separator < 0 || !command || !streamPath || !timelinePath) {
  process.stderr.write(
    "usage: content-mapper-replay-proxy.ts <stream|-> <timeline|-> -- <exec...>\n",
  );
  process.exit(2);
}
const open = (path: string) =>
  path === "-" ? undefined : openSync(path.replaceAll("%p", String(process.pid)), "a");
const stream = open(streamPath);
const timeline = open(timelinePath);
const start = performance.now();
const mapper = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });

/** Frame ids as bytes pass; a frame's body is joined only once complete. */
function frames(dir: "req" | "rep"): (chunk: Buffer) => void {
  let chunks: Buffer[] = [];
  let buffered = 0;
  let need = 0;
  return (chunk) => {
    chunks.push(chunk);
    buffered += chunk.length;
    if (buffered < need || timeline === undefined) {
      if (timeline === undefined) chunks = [];
      return;
    }
    let pending = chunks.length === 1 ? (chunks[0] as Buffer) : Buffer.concat(chunks, buffered);
    let lines = "";
    for (;;) {
      const end = pending.indexOf("\r\n\r\n");
      if (end < 0) break;
      const length = Number(
        /Content-Length:\s*(\d+)/i.exec(pending.toString("latin1", 0, end))?.[1],
      );
      if (pending.length < end + 4 + length) {
        need = end + 4 + length;
        break;
      }
      need = 0;
      const head = pending.toString("utf8", end + 4, end + 4 + Math.min(length, 256));
      const id = /"id":("(?:[^"\\]|\\.)*"|-?\d+|null)/.exec(head)?.[1] ?? "null";
      const method = /"method":"([^"]*)"/.exec(head)?.[1];
      const t = Number((performance.now() - start).toFixed(3));
      lines += `${JSON.stringify({ t, dir, id: JSON.stringify(JSON.parse(id)), method })}\n`;
      pending = pending.subarray(end + 4 + length);
    }
    if (lines) writeSync(timeline, lines);
    chunks = pending.length ? [pending] : [];
    buffered = pending.length;
  };
}

const fromHost = frames("req");
const fromMapper = frames("rep");
process.stdin.on("data", (chunk: Buffer) => {
  if (stream !== undefined) writeSync(stream, chunk);
  fromHost(chunk);
  mapper.stdin.write(chunk);
});
process.stdin.on("end", () => mapper.stdin.end());
// A mapper that already exited reports through "close"; only that write failure is expected.
mapper.stdin.on("error", (error: NodeJS.ErrnoException) => {
  if (error.code !== "EPIPE") throw error;
});
mapper.stdout.on("data", (chunk: Buffer) => {
  fromMapper(chunk);
  process.stdout.write(chunk);
});
for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => mapper.kill(signal));
}
mapper.on("close", (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
