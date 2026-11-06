import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const CAPTURE_LIMIT = 8 * 1024 * 1024;
const FORWARDED_SIGNALS = ["SIGHUP", "SIGINT", "SIGTERM"];

function streamReceipt() {
  const hash = createHash("sha256");
  const chunks = [];
  let bytes = 0;
  let capturedBytes = 0;
  return {
    push(value) {
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
    finish() {
      return {
        bytes,
        capturedBytes,
        complete: bytes === capturedBytes,
        sha256: hash.digest("hex"),
        base64: Buffer.concat(chunks).toString("base64"),
      };
    },
  };
}

const [, , receiptArgument, command, ...args] = process.argv;
assert.ok(receiptArgument && command, "usage: process-receipt.mjs RECEIPT COMMAND [ARG ...]");
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
let spawnError = null;

process.stdin.on("data", (chunk) => streams.stdin.push(chunk));
process.stdin.pipe(child.stdin);
child.stdin.on("error", (error) => {
  if (error.code !== "EPIPE") throw error;
});
child.stdout.on("data", (chunk) => streams.stdout.push(chunk));
child.stdout.pipe(process.stdout, { end: false });
child.stderr.on("data", (chunk) => streams.stderr.push(chunk));
child.stderr.pipe(process.stderr, { end: false });
child.on("error", (error) => {
  spawnError = { name: error.name, message: error.message, code: error.code ?? null };
});

const forward = new Map(
  FORWARDED_SIGNALS.map((signal) => {
    const handler = () => child.kill(signal);
    process.on(signal, handler);
    return [signal, handler];
  }),
);

child.on("close", (code, signal) => {
  const receipt = {
    schema: "vize.contentMapperProcessReceipt.v1",
    command: { file: command, args, cwd: process.cwd() },
    platform: process.platform,
    arch: process.arch,
    node: process.version,
    pid: child.pid ?? null,
    startedAt,
    finishedAt: new Date().toISOString(),
    spawnError,
    exit: { code, signal },
    streams: {
      stdin: streams.stdin.finish(),
      stdout: streams.stdout.finish(),
      stderr: streams.stderr.finish(),
    },
  };
  fs.writeFileSync(receiptFile, `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" });

  for (const [name, handler] of forward) process.removeListener(name, handler);
  if (signal && FORWARDED_SIGNALS.includes(signal)) {
    process.kill(process.pid, signal);
    return;
  }
  process.exitCode = spawnError ? 127 : (code ?? 1);
});
