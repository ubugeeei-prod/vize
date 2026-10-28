import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { parse, record, string, write, type RecordValue, type Runtime } from "./model.ts";
import { environment, verifyCustody } from "./process.ts";
type Waiter = {
  accept: (packet: RecordValue) => boolean;
  resolve: (packet: RecordValue) => void;
  reject: (error: Error) => void;
  timer: NodeJS.Timeout;
};
type Pending = {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  timer: NodeJS.Timeout;
};
export class PublicRefSession {
  private readonly child: ChildProcessWithoutNullStreams;
  private readonly pending = new Map<number, Pending>();
  private readonly waiters: Waiter[] = [];
  private readonly backlog: RecordValue[] = [];
  private buffer = Buffer.alloc(0);
  private id = 0;
  private closing = false;
  private initialized = false;
  private readonly runtime: Runtime;
  readonly directory: string;
  readonly output: string;
  private failure: Error | undefined;
  private readonly closed: Promise<void>;
  readonly messages: RecordValue[] = [];
  constructor(runtime: Runtime, directory: string, output: string) {
    this.runtime = runtime;
    this.directory = directory;
    this.output = output;
    fs.mkdirSync(output, { recursive: true });
    this.child = spawn(runtime.node, [runtime.cli, "lsp"], {
      cwd: directory,
      env: environment(runtime, path.join(output, "native")),
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.closed = new Promise((resolve) => this.child.once("close", () => resolve()));
    write(path.join(output, "launch.txt"), {
      command: [runtime.node, runtime.cli, "lsp"],
      cwd: directory,
      pid: this.child.pid,
      native: runtime.native,
    });
    this.child.stdout.on("data", (bytes: Buffer) => {
      fs.appendFileSync(path.join(output, "server-wire.txt"), bytes);
      try {
        this.buffer = Buffer.concat([this.buffer, bytes]);
        this.drain();
      } catch (error) {
        this.fail(error);
      }
    });
    this.child.stderr.on("data", (bytes: Buffer) =>
      fs.appendFileSync(path.join(output, "stderr.txt"), bytes),
    );
    this.child.on("error", (error) => this.fail(error));
    this.child.on("exit", (code, signal) => {
      write(path.join(output, "exit.txt"), { code, signal });
      if (!this.closing || code !== 0 || signal !== null)
        this.fail(new Error("public LSP exited unexpectedly"));
    });
  }
  private fail(value: unknown): void {
    const error = value instanceof Error ? value : new Error(String(value));
    this.failure ??= error;
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
    for (const waiter of this.waiters) {
      clearTimeout(waiter.timer);
      waiter.reject(error);
    }
    this.waiters.length = 0;
  }
  private send(message: RecordValue): void {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    const frame = Buffer.from("Content-Length: " + Buffer.byteLength(body) + "\r\n\r\n" + body);
    fs.appendFileSync(path.join(this.output, "client-wire.txt"), frame);
    this.child.stdin.write(frame);
  }
  notify(method: string, params: unknown): void {
    if (this.failure) throw this.failure;
    this.send({ method, params });
  }
  request(method: string, params: unknown): Promise<unknown> {
    if (this.failure) return Promise.reject(this.failure);
    const id = ++this.id;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        const error = new Error("public LSP request timeout: " + method);
        this.fail(error);
        reject(error);
      }, 120_000);
      this.pending.set(id, { resolve, reject, timer });
      this.send({ id, method, params });
    });
  }
  private drain(): void {
    for (;;) {
      const end = this.buffer.indexOf("\r\n\r\n");
      if (end < 0) return;
      const header = new TextDecoder("utf8", { fatal: true }).decode(this.buffer.subarray(0, end));
      const m = /^Content-Length:\s*(\d+)$/imu.exec(header);
      assert.ok(m && m[1]);
      const length = Number(m[1]);
      assert.ok(Number.isSafeInteger(length) && length >= 0);
      if (this.buffer.length < end + 4 + length) return;
      const message = record(parse(this.buffer.subarray(end + 4, end + 4 + length)));
      this.buffer = this.buffer.subarray(end + 4 + length);
      this.messages.push(message);
      if (typeof message.id === "number" && message.method === undefined) {
        const pending = this.pending.get(message.id);
        assert.ok(pending, "unowned public LSP response");
        this.pending.delete(message.id);
        clearTimeout(pending.timer);
        if (message.error !== undefined) pending.reject(new Error(JSON.stringify(message.error)));
        else pending.resolve(message.result);
      } else if (typeof message.id === "number" && typeof message.method === "string") {
        this.send({
          id: message.id,
          error: { code: -32601, message: "client does not implement " + message.method },
        });
      } else if (typeof message.method === "string") {
        const index = this.waiters.findIndex((waiter) => waiter.accept(message));
        if (index < 0) this.backlog.push(message);
        else {
          const [waiter] = this.waiters.splice(index, 1);
          assert.ok(waiter);
          clearTimeout(waiter.timer);
          waiter.resolve(message);
        }
      } else throw new Error("unclassified public LSP envelope");
    }
  }
  diagnostics(uri: string, version: number): Promise<RecordValue> {
    const accept = (packet: RecordValue) => {
      if (packet.method !== "textDocument/publishDiagnostics") return false;
      const params = record(packet.params);
      return params.uri === uri && params.version === version;
    };
    const index = this.backlog.findIndex(accept);
    if (index >= 0) {
      const [packet] = this.backlog.splice(index, 1);
      assert.ok(packet);
      return Promise.resolve(packet);
    }
    if (this.failure) return Promise.reject(this.failure);
    return new Promise((resolve, reject) => {
      const waiter: Waiter = {
        accept,
        resolve,
        reject,
        timer: setTimeout(() => {
          const i = this.waiters.indexOf(waiter);
          if (i >= 0) this.waiters.splice(i, 1);
          const error = new Error("public diagnostic timeout: " + uri + "@" + version);
          this.fail(error);
          reject(error);
        }, 120_000),
      };
      this.waiters.push(waiter);
    });
  }
  async initialize(): Promise<void> {
    const rootUri = pathToFileURL(this.directory).href;
    const result = record(
      await this.request("initialize", {
        processId: process.pid,
        rootUri,
        workspaceFolders: [{ uri: rootUri, name: "type-admission" }],
        capabilities: { textDocument: { hover: { contentFormat: ["markdown", "plaintext"] } } },
        initializationOptions: { editor: true, hover: true, typecheck: true, lint: false },
      }),
    );
    assert.equal(string(record(result.serverInfo).version), this.runtime.version);
    this.notify("initialized", {});
    this.initialized = true;
  }
  async shutdown(): Promise<unknown[]> {
    if (!this.failure && this.initialized) {
      assert.equal(await this.request("shutdown", null), null);
      this.closing = true;
      this.notify("exit", null);
    } else {
      this.closing = true;
      this.child.kill();
    }
    let timer: NodeJS.Timeout | undefined;
    await Promise.race([
      this.closed,
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => {
          this.child.kill();
          reject(new Error("public LSP did not exit"));
        }, 10_000);
      }),
    ]).finally(() => clearTimeout(timer));
    write(path.join(this.output, "complete-envelopes.txt"), this.messages);
    assert.equal(this.buffer.length, 0, "truncated final LSP frame");
    assert.equal(this.child.exitCode, 0);
    assert.equal(this.child.signalCode, null);
    if (this.failure) throw this.failure;
    return verifyCustody(this.runtime, path.join(this.output, "native"));
  }
}
