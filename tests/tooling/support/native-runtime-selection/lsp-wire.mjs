import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { decodeNativeFrames } from "./lsp-strace.mjs";

const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** Own the actual straced source process and retain every original outer byte. */
export class SourceLspWire {
  constructor({ executable, strace, environment, workspace, directory }) {
    fs.mkdirSync(directory, { recursive: true });
    this.directory = directory;
    this.prefix = path.join(directory, "syscalls");
    this.argv = [
      "-ff",
      "-q",
      "-ttt",
      "-T",
      "-xx",
      "-s",
      "16777216",
      "-e",
      "trace=process,read,write,readv,writev",
      "-o",
      this.prefix,
      executable,
      "lsp",
      "--stdio",
    ];
    this.events = [];
    this.input = [];
    this.output = [];
    this.errors = [];
    this.pending = Buffer.alloc(0);
    this.waiters = new Set();
    this.status = null;
    this.failure = null;
    this.nextId = 0;
    this.child = spawn(strace, this.argv, {
      cwd: workspace,
      env: environment,
      stdio: "pipe",
      detached: true,
    });
    this.closed = new Promise((done) =>
      this.child.once("close", (code, signal) => {
        this.status = { code, signal };
        this.events.push({ kind: "process-closed", status: this.status });
        this.wake();
        done(this.status);
      }),
    );
    this.child.on("error", (error) => {
      this.failure = String(error);
      this.wake();
    });
    this.child.stdin.on("error", (error) => {
      this.events.push({ kind: "stdin-error", error: String(error) });
      this.wake();
    });
    this.child.stderr.on("data", (bytes) => this.errors.push(Buffer.from(bytes)));
    this.child.stdout.on("data", (bytes) => {
      this.output.push(Buffer.from(bytes));
      this.pending = Buffer.concat([this.pending, bytes]);
      try {
        this.parse();
      } catch (error) {
        this.failure = String(error);
      }
      this.wake();
    });
  }
  wake() {
    for (const callback of this.waiters) callback();
  }
  parse() {
    while (true) {
      const boundary = this.pending.indexOf("\r\n\r\n");
      if (boundary < 0) return;
      const lengths = this.pending
        .subarray(0, boundary)
        .toString("ascii")
        .split("\r\n")
        .filter((header) => /^content-length:/iu.test(header));
      assert.equal(lengths.length, 1);
      const length = Number(lengths[0].slice(lengths[0].indexOf(":") + 1));
      assert.ok(Number.isSafeInteger(length) && length >= 0 && length <= 16 * 1024 * 1024);
      const end = boundary + 4 + length;
      if (this.pending.length < end) return;
      const [{ packet }] = decodeNativeFrames(this.pending.subarray(0, end));
      this.pending = this.pending.subarray(end);
      this.events.push({ direction: "server", packet });
      if (packet.method && packet.id !== undefined)
        this.send({
          jsonrpc: "2.0",
          id: packet.id,
          result:
            packet.method === "workspace/configuration"
              ? (packet.params?.items ?? []).map(() => null)
              : null,
        });
    }
  }
  send(packet) {
    const body = Buffer.from(JSON.stringify(packet));
    const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
    this.input.push(frame);
    this.events.push({ direction: "client", packet });
    this.child.stdin.write(frame);
  }
  notify(method, params) {
    this.send({ jsonrpc: "2.0", method, ...(params === undefined ? {} : { params }) });
  }
  async request(method, params) {
    const id = ++this.nextId;
    this.send({ jsonrpc: "2.0", id, method, ...(params === undefined ? {} : { params }) });
    return this.wait((packet) => packet.id === id && !packet.method, method);
  }
  async wait(predicate, operation, from = 0, timeout = 20000) {
    const deadline = performance.now() + timeout;
    while (true) {
      const found = this.events
        .slice(from)
        .find((event) => event.direction === "server" && predicate(event.packet));
      if (found) return found.packet;
      assert.equal(this.failure, null, this.failure ?? "outer wire remains valid");
      assert.equal(this.status, null, `source process exited before ${operation}`);
      const remaining = deadline - performance.now();
      assert.ok(remaining > 0, `${operation} timed out`);
      await new Promise((done, reject) => {
        const timer = setTimeout(() => {
          this.waiters.delete(wake);
          reject(new Error(`${operation} timed out`));
        }, remaining);
        const wake = () => {
          clearTimeout(timer);
          this.waiters.delete(wake);
          done();
        };
        this.waiters.add(wake);
      });
    }
  }
  async end() {
    this.child.stdin.end();
    let timer;
    try {
      await Promise.race([
        this.closed,
        new Promise((_, reject) => {
          timer = setTimeout(
            () => reject(new Error("owned source/trace process did not close")),
            10000,
          );
        }),
      ]);
    } finally {
      clearTimeout(timer);
      if (!this.status) {
        this.forcedTermination = true;
        process.kill(-this.child.pid, "SIGKILL");
        await this.closed;
      }
      this.save();
    }
  }
  save() {
    const streams = {};
    for (const [name, chunks] of Object.entries({
      client: this.input,
      server: this.output,
      stderr: this.errors,
    })) {
      const bytes = Buffer.concat(chunks),
        destination = path.join(this.directory, `${name}.bin`);
      fs.writeFileSync(destination, bytes);
      const metadata = { path: destination, bytes: bytes.length, sha256: sha(bytes) };
      if (name !== "stderr") {
        try {
          metadata.packets = decodeNativeFrames(bytes);
          metadata.framing = "complete";
        } catch (error) {
          metadata.framing = "invalid";
          metadata.framingError = String(error);
        }
      }
      streams[name] = metadata;
    }
    const observation = {
      pid: this.child.pid,
      argv: this.argv,
      status: this.status,
      forcedTermination: this.forcedTermination ?? false,
      pendingBytes: this.pending.length,
      failure: this.failure,
      streams,
      events: this.events,
    };
    fs.writeFileSync(
      path.join(this.directory, "outer-observation.json"),
      JSON.stringify(observation, null, 2) + "\n",
    );
    return observation;
  }
}
