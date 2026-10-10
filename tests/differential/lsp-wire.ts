import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { errorText, type JsonRpcMessage, type WireObservation } from "./lsp-types.ts";

const MAX_BYTES = 16 * 1024 * 1024;

export function frameMessage(message: unknown): Buffer {
  const body = Buffer.from(JSON.stringify(message));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
}

// Keep every received byte. Parsing is only an index into the captured transport;
// the complete frame, order and omitted JSON properties remain observable.
export function decodeFrames(
  bytes: Buffer,
  complete = true,
): { messages: JsonRpcMessage[]; consumed: number } {
  assert(Buffer.isBuffer(bytes), "raw framed bytes are required");
  assert(bytes.length <= MAX_BYTES, "LSP wire observation exceeds its bound");
  const messages: JsonRpcMessage[] = [];
  let offset = 0;
  while (offset < bytes.length) {
    const separator = bytes.indexOf("\r\n\r\n", offset);
    if (separator < 0) break;
    const header = bytes.subarray(offset, separator).toString("ascii");
    const lengths = [...header.matchAll(/^Content-Length: (\d+)$/gim)];
    assert.equal(lengths.length, 1, "one Content-Length is required");
    const length = Number(lengths[0][1]);
    assert(Number.isSafeInteger(length) && length <= MAX_BYTES, "invalid frame length");
    const end = separator + 4 + length;
    if (end > bytes.length) break;
    const body = new TextDecoder("utf-8", { fatal: true }).decode(
      bytes.subarray(separator + 4, end),
    );
    const message = JSON.parse(body) as JsonRpcMessage;
    assert.equal(message.jsonrpc, "2.0", "JSON-RPC 2.0 envelope is required");
    messages.push(message);
    offset = end;
  }
  if (complete) assert.equal(offset, bytes.length, "truncated LSP frame");
  return { messages, consumed: offset };
}

type Waiter = {
  resolve: (message: JsonRpcMessage) => void;
  reject: (error: Error) => void;
  timeout: ReturnType<typeof setTimeout>;
};
type NotificationWaiter = Waiter & { predicate: (message: JsonRpcMessage) => boolean };

export class LspWire {
  clientChunks: Buffer[] = [];
  serverChunks: Buffer[] = [];
  stderrChunks: Buffer[] = [];
  messages: JsonRpcMessage[] = [];
  processError: string | null = null;
  exitStatus: number | null = null;
  signal: string | null = null;
  nextId = 0;
  pending = new Map<number, Waiter>();
  waiters: NotificationWaiter[] = [];
  buffer = Buffer.alloc(0);
  serverBytes = 0;
  child: ChildProcessWithoutNullStreams;
  exited: Promise<void>;

  constructor(command: string, args: string[], cwd: string, env?: NodeJS.ProcessEnv) {
    this.child = spawn(command, args, { cwd, env, stdio: ["pipe", "pipe", "pipe"] });
    this.exited = new Promise<void>((resolve) => {
      this.child.once("close", (code, signal) => {
        this.exitStatus = code;
        this.signal = signal;
        this.fail(new Error(`LSP exited (${code}, ${signal})`));
        resolve();
      });
    });
    this.child.on("error", (error) => {
      this.processError = error.message;
      this.fail(error);
    });
    this.child.stdin.on("error", (error) => this.fail(error));
    this.child.stderr.on("data", (chunk: Buffer) => this.stderrChunks.push(Buffer.from(chunk)));
    this.child.stdout.on("data", (chunk: Buffer) => {
      this.serverBytes += chunk.length;
      this.serverChunks.push(Buffer.from(chunk));
      this.buffer = Buffer.concat([this.buffer, chunk]);
      try {
        assert(this.serverBytes <= MAX_BYTES, "server wire observation exceeds its bound");
        const decoded = decodeFrames(this.buffer, false);
        this.buffer = this.buffer.subarray(decoded.consumed);
        for (const message of decoded.messages) this.receive(message);
      } catch (error) {
        this.processError = errorText(error);
        this.fail(new Error(this.processError));
        this.child.kill();
      }
    });
  }

  fail(error: Error): void {
    for (const waiter of [...this.pending.values(), ...this.waiters]) {
      clearTimeout(waiter.timeout);
      waiter.reject(error);
    }
    this.pending.clear();
    this.waiters.length = 0;
  }

  receive(message: JsonRpcMessage): void {
    this.messages.push(message);
    if (typeof message.id === "number" && !Object.hasOwn(message, "method")) {
      const pending = this.pending.get(message.id);
      if (pending) {
        clearTimeout(pending.timeout);
        this.pending.delete(message.id);
        pending.resolve(message);
      }
    }
    // A matched waiter is removed while iterating, so retain a separate snapshot.
    for (const waiter of this.waiters.slice()) {
      if (waiter.predicate(message)) {
        clearTimeout(waiter.timeout);
        this.waiters.splice(this.waiters.indexOf(waiter), 1);
        waiter.resolve(message);
      }
    }
  }

  send(message: JsonRpcMessage): void {
    const bytes = frameMessage(message);
    this.clientChunks.push(bytes);
    assert(Buffer.concat(this.clientChunks).length <= MAX_BYTES, "client wire exceeds bound");
    this.child.stdin.write(bytes);
  }

  notify(method: string, params?: Record<string, unknown>): void {
    this.send({ jsonrpc: "2.0", method, ...(params === undefined ? {} : { params }) });
  }

  request(
    method: string,
    params?: Record<string, unknown>,
    timeoutMs = 30_000,
  ): Promise<JsonRpcMessage> {
    const id = ++this.nextId;
    const response = new Promise<JsonRpcMessage>((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Timed out waiting for ${method}`));
      }, timeoutMs);
      this.pending.set(id, { resolve, reject, timeout });
    });
    this.send({ jsonrpc: "2.0", id, method, ...(params === undefined ? {} : { params }) });
    return response;
  }

  waitFor(
    predicate: (message: JsonRpcMessage) => boolean,
    timeoutMs = 30_000,
  ): Promise<JsonRpcMessage> {
    const observed = this.messages.find(predicate);
    if (observed) return Promise.resolve(observed);
    return new Promise<JsonRpcMessage>((resolve, reject) => {
      const waiter: NotificationWaiter = {
        predicate,
        resolve,
        reject,
        timeout: setTimeout(() => {
          this.waiters.splice(this.waiters.indexOf(waiter), 1);
          reject(new Error("Timed out waiting for exact-version diagnostics"));
        }, timeoutMs),
      };
      this.waiters.push(waiter);
    });
  }

  async finish(): Promise<void> {
    const shutdown = await this.request("shutdown", undefined, 10_000);
    assert.deepEqual(shutdown, { jsonrpc: "2.0", id: this.nextId, result: null });
    this.notify("exit");
    this.child.stdin.end();
    await this.stop(false);
    assert.equal(this.exitStatus, 0, "LSP must shut down successfully");
    assert.equal(this.signal, null, "LSP must exit without a signal");
    assert.equal(this.processError, null, "LSP transport must not fail");
  }

  async stop(force = true): Promise<void> {
    if (force && this.child.exitCode === null && this.child.signalCode === null) {
      this.child.kill();
    }
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      await Promise.race([
        this.exited,
        new Promise((_, reject) => {
          timeout = setTimeout(() => {
            this.child.kill("SIGKILL");
            reject(new Error("LSP did not exit before the cleanup deadline"));
          }, 5_000);
        }),
      ]);
    } finally {
      clearTimeout(timeout);
    }
  }

  observation(): WireObservation {
    return {
      clientWireBase64: Buffer.concat(this.clientChunks).toString("base64"),
      serverWireBase64: Buffer.concat(this.serverChunks).toString("base64"),
      stderrBase64: Buffer.concat(this.stderrChunks).toString("base64"),
      exitStatus: this.exitStatus,
      signal: this.signal,
      processError: this.processError,
    };
  }
}
