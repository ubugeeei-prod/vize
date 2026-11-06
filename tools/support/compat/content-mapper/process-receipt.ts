import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const CAPTURE_LIMIT = 8 * 1024 * 1024;
const FORWARDED_SIGNALS = ["SIGHUP", "SIGINT", "SIGTERM"] as const;

function streamReceipt(): {
  push(value: Uint8Array): void;
  snapshot(): {
    bytes: number;
    capturedBytes: number;
    complete: boolean;
    sha256: string;
    base64: string;
    utf8: string;
  };
} {
  const hash = createHash("sha256");
  const chunks: Buffer[] = [];
  let bytes = 0;
  let capturedBytes = 0;
  return {
    push(value: Uint8Array) {
      const chunk = Buffer.from(value);
      bytes += chunk.length;
      hash.update(chunk);
      const available = CAPTURE_LIMIT - capturedBytes;
      if (available > 0) {
        const captured = chunk.subarray(0, available);
        chunks.push(captured);
        capturedBytes += captured.length;
      }
    },
    snapshot() {
      return {
        bytes,
        capturedBytes,
        complete: bytes === capturedBytes,
        sha256: hash.copy().digest("hex"),
        base64: Buffer.concat(chunks).toString("base64"),
        utf8: Buffer.concat(chunks).toString("utf8"),
      };
    },
  };
}

const [, , receiptArgument, command, ...args] = process.argv;
assert.ok(receiptArgument && command, "usage: process-receipt.ts RECEIPT COMMAND [ARG ...]");
const receiptFile = path.resolve(receiptArgument);
assert.equal(fs.existsSync(receiptFile), false, `receipt already exists: ${receiptFile}`);
fs.mkdirSync(path.dirname(receiptFile), { recursive: true });

const streams = {
  stdin: streamReceipt(),
  stdout: streamReceipt(),
  stderr: streamReceipt(),
};
const startedAt = new Date().toISOString();
const child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
let lifecycle: "running" | "closed" = "running";
let finishedAt: string | null = null;
let spawnError: { name: string; message: string; code: string | null } | null = null;
let exit: { code: number | null; signal: NodeJS.Signals | null } | null = null;

// The TypeScript host closes a healthy mapper with SIGKILL. Persist each protocol
// transition before forwarding it so a successful host teardown cannot erase the
// evidence. A mapper that exits by itself reaches the final `closed` snapshot.
function persist(): void {
  const receipt = {
    schema: "vize.contentMapperProcessReceipt.v1",
    command: { file: command, args, cwd: process.cwd() },
    platform: process.platform,
    arch: process.arch,
    node: process.version,
    pid: child.pid ?? null,
    startedAt,
    finishedAt,
    lifecycle,
    spawnError,
    exit,
    streams: {
      stdin: streams.stdin.snapshot(),
      stdout: streams.stdout.snapshot(),
      stderr: streams.stderr.snapshot(),
    },
  };
  const temporary = `${receiptFile}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify(receipt, null, 2)}\n`);
  fs.renameSync(temporary, receiptFile);
}

persist();

process.stdin.on("data", (chunk: Buffer) => {
  streams.stdin.push(chunk);
  persist();
});
process.stdin.pipe(child.stdin);
child.stdin.on("error", (error: NodeJS.ErrnoException) => {
  if (error.code !== "EPIPE") throw error;
});
child.stdout.on("data", (chunk: Buffer) => {
  streams.stdout.push(chunk);
  persist();
});
child.stdout.pipe(process.stdout, { end: false });
child.stderr.on("data", (chunk: Buffer) => {
  streams.stderr.push(chunk);
  persist();
});
child.stderr.pipe(process.stderr, { end: false });
child.on("error", (error) => {
  const code = (error as NodeJS.ErrnoException).code ?? null;
  spawnError = { name: error.name, message: error.message, code };
  persist();
});

const forward = new Map(
  FORWARDED_SIGNALS.map((signal) => {
    const handler = () => child.kill(signal);
    process.on(signal, handler);
    return [signal, handler];
  }),
);

child.on("close", (code, signal) => {
  lifecycle = "closed";
  finishedAt = new Date().toISOString();
  exit = { code, signal };
  persist();

  for (const [name, handler] of forward) process.removeListener(name, handler);
  if (signal && FORWARDED_SIGNALS.includes(signal)) {
    process.kill(process.pid, signal);
    return;
  }
  process.exitCode = spawnError ? 127 : (code ?? 1);
});
