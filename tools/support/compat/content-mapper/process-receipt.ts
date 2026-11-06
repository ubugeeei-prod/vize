import assert from "node:assert/strict";
import { spawn, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pipeline } from "node:stream";

const CAPTURE_LIMIT = 8 * 1024 * 1024;
const FORWARDED_SIGNALS = ["SIGHUP", "SIGINT", "SIGTERM"] as const;
const OBSERVER_MODE = "--observe-child";

type ProcessError = { name: string; message: string; code: string | null };

function processError(error: Error): ProcessError {
  return {
    name: error.name,
    message: error.message,
    code: (error as NodeJS.ErrnoException).code ?? null,
  };
}

function forwardSignals(child: ChildProcess): () => void {
  const handlers = new Map(
    FORWARDED_SIGNALS.map((signal) => {
      const handler = () => child.kill(signal);
      process.on(signal, handler);
      return [signal, handler] as const;
    }),
  );
  return () => {
    for (const [signal, handler] of handlers) process.removeListener(signal, handler);
  };
}

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

/** Relay the host protocol through an observer that survives a host SIGKILL. */
function relay(parameters: string[]): void {
  const [receiptArgument, command, ...args] = parameters;
  assert.ok(receiptArgument && command, "usage: process-receipt.ts RECEIPT COMMAND [ARG ...]");
  const script = path.resolve(process.argv[1]!);
  const observer = spawn(
    process.execPath,
    [script, OBSERVER_MODE, receiptArgument, command, ...args],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  let observerError: ProcessError | null = null;
  const stopForwarding = forwardSignals(observer);
  pipeline(process.stdin, observer.stdin, (error) => {
    if (error && (error as NodeJS.ErrnoException).code !== "EPIPE") {
      observerError = processError(error);
      process.stderr.write(`mapper observer stdin: ${error.message}\n`);
    }
  });
  observer.stdout.pipe(process.stdout, { end: false });
  observer.stderr.pipe(process.stderr, { end: false });
  observer.on("error", (error) => {
    observerError = processError(error);
    process.stderr.write(`mapper observer startup: ${error.message}\n`);
  });
  observer.on("close", (code, signal) => {
    stopForwarding();
    if (signal && (FORWARDED_SIGNALS as readonly NodeJS.Signals[]).includes(signal)) {
      process.kill(process.pid, signal);
      return;
    }
    process.exitCode = observerError ? 127 : (code ?? 1);
  });
}

/** Own the mapper child and durably retain its final state after relay teardown. */
function observe(parameters: string[]): void {
  const [receiptArgument, command, ...args] = parameters;
  assert.ok(receiptArgument && command, "observer requires RECEIPT COMMAND [ARG ...]");
  const receiptFile = path.resolve(receiptArgument);
  assert.equal(fs.existsSync(receiptFile), false, `receipt already exists: ${receiptFile}`);
  fs.mkdirSync(path.dirname(receiptFile), { recursive: true });

  const streams = {
    stdin: streamReceipt(),
    stdout: streamReceipt(),
    stderr: streamReceipt(),
  };
  const transport: {
    stdin: { ended: boolean; error: ProcessError | null };
    stdout: { error: ProcessError | null };
    stderr: { error: ProcessError | null };
  } = {
    stdin: { ended: false, error: null },
    stdout: { error: null },
    stderr: { error: null },
  };
  const startedAt = new Date().toISOString();
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
  let lifecycle: "running" | "closed" = "running";
  let finishedAt: string | null = null;
  let spawnError: ProcessError | null = null;
  let exit: { code: number | null; signal: NodeJS.Signals | null } | null = null;

  // The host can SIGKILL the relay. The observer persists every protocol
  // transition and then records the actual mapper's closure after relay EOF.
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
      transport,
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
  pipeline(process.stdin, child.stdin, (error) => {
    transport.stdin.ended = process.stdin.readableEnded;
    if (error && (error as NodeJS.ErrnoException).code !== "EPIPE")
      transport.stdin.error = processError(error);
    persist();
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
  for (const [stream, name] of [
    [process.stdout, "stdout"],
    [process.stderr, "stderr"],
  ] as const)
    stream.on("error", (error) => {
      transport[name].error = processError(error);
      persist();
    });
  child.on("error", (error) => {
    spawnError = processError(error);
    persist();
  });

  const stopForwarding = forwardSignals(child);
  child.on("close", (code, signal) => {
    lifecycle = "closed";
    finishedAt = new Date().toISOString();
    exit = { code, signal };
    persist();
    stopForwarding();
    if (signal && (FORWARDED_SIGNALS as readonly NodeJS.Signals[]).includes(signal)) {
      process.kill(process.pid, signal);
      return;
    }
    process.exitCode = spawnError ? 127 : (code ?? 1);
  });
}

if (process.argv[2] === OBSERVER_MODE) observe(process.argv.slice(3));
else relay(process.argv.slice(2));
