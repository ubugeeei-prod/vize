#!/usr/bin/env node
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Event = Record<string, unknown>;
const target = process.env.VIZE_UI_RPC_REAL_EXE;
const logPath = process.env.VIZE_UI_RPC_LOG;
if (!target || !logPath) throw Error("explicit observer target and log are required");
const stamp = () => process.hrtime.bigint().toString();
const rawPrefix = process.env.VIZE_UI_RPC_CAPTURE_PREFIX;
const logFd = fs.openSync(logPath, "a");
const captures = rawPrefix
  ? {
      outgoing: fs.openSync(rawPrefix + "-" + process.pid + "-to-go.wire", "a"),
      incoming: fs.openSync(rawPrefix + "-" + process.pid + "-from-go.wire", "a"),
    }
  : null;
const log = (event: Event) => {
  fs.writeSync(logFd, JSON.stringify({ proxyPid: process.pid, ns: stamp(), ...event }) + "\n");
};
class WireId {
  raw: string;
  constructor(raw: string) {
    this.raw = raw;
  }
}
function metadata(prefix: Buffer, complete: boolean): Event {
  if (!complete)
    return { decoded: false, reason: "body exceeds metadata capture; bytes still forwarded" };
  try {
    const raw: unknown = JSON.parse(
      prefix.toString("utf8"),
      (key: string, value: unknown, context?: { source?: string }) =>
        key === "id" && typeof value === "number"
          ? context?.source
            ? new WireId(context.source)
            : null
          : value,
    );
    if (!raw || typeof raw !== "object" || Array.isArray(raw))
      return { decoded: false, reason: "nonobject frame" };
    const row = raw as Record<string, unknown>;
    const id =
      row.id instanceof WireId
        ? "n:" + row.id.raw
        : typeof row.id === "string"
          ? "s:" + row.id
          : null;
    return {
      decoded: true,
      id,
      method: typeof row.method === "string" ? row.method : null,
      response: !("method" in row) && "id" in row,
      hasError: "error" in row,
    };
  } catch {
    return { decoded: false, reason: "non-JSON frame" };
  }
}
class Frames {
  direction: string;
  header = Buffer.alloc(0);
  remaining: number | null = null;
  length = 0;
  prefix: Buffer[] = [];
  prefixBytes = 0;
  firstNs = "";
  digest = createHash("sha256");
  unavailable = false;
  constructor(direction: string) {
    this.direction = direction;
  }
  push(chunk: Buffer) {
    if (this.unavailable) return;
    let cursor = 0;
    while (cursor < chunk.length) {
      if (this.remaining === null) {
        const all = Buffer.concat([this.header, chunk.subarray(cursor)]);
        const end = all.indexOf("\r\n\r\n");
        if (end < 0) {
          if (all.length > 16_384) {
            this.unavailable = true;
            log({ event: "framing-unavailable", direction: this.direction });
            return;
          }
          this.header = all;
          return;
        }
        const match = /(?:^|\r\n)Content-Length:\s*(\d+)(?:\r\n|$)/i.exec(
          all.subarray(0, end).toString("ascii"),
        );
        const length = match ? Number(match[1]) : NaN;
        if (!Number.isSafeInteger(length) || length < 0) {
          this.unavailable = true;
          log({ event: "framing-unavailable", direction: this.direction });
          return;
        }
        cursor += end + 4 - this.header.length;
        this.header = Buffer.alloc(0);
        this.remaining = this.length = length;
        this.prefix = [];
        this.prefixBytes = 0;
        this.firstNs = stamp();
        this.digest = createHash("sha256");
      }
      const take = Math.min(this.remaining, chunk.length - cursor);
      const piece = chunk.subarray(cursor, cursor + take);
      this.digest.update(piece);
      const capture = Math.min(take, 262_144 - this.prefixBytes);
      if (capture > 0) {
        this.prefix.push(Buffer.from(piece.subarray(0, capture)));
        this.prefixBytes += capture;
      }
      cursor += take;
      this.remaining -= take;
      if (this.remaining === 0) {
        log({
          event: "frame",
          direction: this.direction,
          firstNs: this.firstNs,
          payloadBytes: this.length,
          sha256: this.digest.digest("hex"),
          ...metadata(Buffer.concat(this.prefix), this.prefixBytes === this.length),
        });
        this.remaining = null;
      }
    }
  }
}
const child = spawn(target, process.argv.slice(2), {
  cwd: process.cwd(),
  env: process.env,
  stdio: ["pipe", "pipe", "pipe"],
});
log({
  event: "spawn",
  parentPid: process.ppid,
  goPid: child.pid ?? null,
  target,
  argv: process.argv.slice(2),
  cwd: process.cwd(),
  modeByPath: "proxy .js retains AsyncJsonRpc classification",
});
const outgoing = new Frames("to-go"),
  incoming = new Frames("from-go");
process.stdin.on("data", (chunk: Buffer) => {
  if (!child.stdin.write(chunk)) process.stdin.pause();
  if (captures) fs.writeSync(captures.outgoing, chunk);
  outgoing.push(chunk);
});
child.stdin.on("drain", () => process.stdin.resume());
process.stdin.on("end", () => child.stdin.end());
child.stdout.on("data", (chunk: Buffer) => {
  if (!process.stdout.write(chunk)) child.stdout.pause();
  if (captures) fs.writeSync(captures.incoming, chunk);
  incoming.push(chunk);
});
process.stdout.on("drain", () => child.stdout.resume());
let stderrBytes = 0;
let stderrPrefix = Buffer.alloc(0);
child.stderr.on("data", (chunk: Buffer) => {
  stderrBytes += chunk.length;
  if (stderrPrefix.length < 8192)
    stderrPrefix = Buffer.concat([stderrPrefix, chunk.subarray(0, 8192 - stderrPrefix.length)]);
  process.stderr.write(chunk);
});
child.on("error", (error: Error) => {
  log({ event: "spawn-error", message: error.message });
  process.exitCode = 1;
});
child.on("close", (code, signal) => {
  log({
    event: "exit",
    goPid: child.pid ?? null,
    code,
    signal,
    stderrBytes,
    stderrPrefix: stderrPrefix.toString("utf8"),
  });
  process.exitCode = code ?? 1;
});
for (const signal of ["SIGTERM", "SIGINT"] as const)
  process.on(signal, () => {
    log({ event: "observer-signal", signal });
    child.kill(signal);
  });
child.stdin.on("error", (error: Error) => log({ event: "stdin-error", message: error.message }));
process.stdout.on("error", (error: Error) =>
  log({ event: "stdout-error", message: error.message }),
);
