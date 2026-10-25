import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const maximumFrameBytes = 16 * 1024 * 1024;
export function takeFrame(bytes) {
  const boundary = bytes.indexOf("\r\n\r\n");
  if (boundary < 0) return null;
  const header = bytes.subarray(0, boundary);
  assert.ok(
    [...header].every((byte) => byte < 128),
    "ASCII protocol header",
  );
  const lengths = header
    .toString("ascii")
    .split("\r\n")
    .filter((line) => /^content-length:/iu.test(line));
  assert.equal(lengths.length, 1);
  assert.match(lengths[0], /^Content-Length: [0-9]+$/iu);
  const length = Number(lengths[0].slice(lengths[0].indexOf(":") + 1));
  assert.ok(Number.isSafeInteger(length) && length >= 0 && length <= maximumFrameBytes);
  const end = boundary + 4 + length;
  if (bytes.length < end) return null;
  const body = bytes.subarray(boundary + 4, end);
  const text = body.toString("utf8");
  assert.deepEqual(Buffer.from(text), body, "lossless native UTF8");
  return { bytes: end, bodySha256: sha256(body), packet: JSON.parse(text) };
}
export function completeFrames(bytes) {
  const frames = [];
  let offset = 0;
  while (offset < bytes.length) {
    const frame = takeFrame(bytes.subarray(offset));
    assert.ok(frame, "complete original native frame");
    frames.push({ offset, ...frame });
    offset += frame.bytes;
  }
  return frames;
}

/** Direct SDK child; event times are auditor receipt times, not syscall timings. */
export class DirectSdkWire {
  constructor({ executable, environment, workspace, directory }) {
    fs.mkdirSync(directory, { recursive: true });
    Object.assign(this, { executable, directory, workspace });
    this.argv = ["--lsp", "--stdio"];
    this.events = [];
    this.input = [];
    this.output = [];
    this.stderr = [];
    this.pending = Buffer.alloc(0);
    this.waiters = new Set();
    this.status = null;
    this.failure = null;
    this.nextId = 0;
    this.child = spawn(executable, this.argv, { cwd: workspace, env: environment, stdio: "pipe" });
    this.mark({ kind: "spawn", pid: this.child.pid, executable, argv: this.argv });
    this.closed = new Promise((resolve) =>
      this.child.once("close", (code, signal) => {
        this.status = { code, signal };
        this.mark({ kind: "process-close", ...this.status });
        this.wake();
        resolve(this.status);
      }),
    );
    this.child.on("exit", (code, signal) => this.mark({ kind: "process-exit", code, signal }));
    this.child.on("error", (error) => {
      this.failure = String(error);
      this.mark({ kind: "process-error", error: String(error) });
      this.wake();
    });
    this.child.stdin.on("error", (error) => {
      this.failure = String(error);
      this.mark({ kind: "stdin-error", error: String(error) });
      this.wake();
    });
    this.child.stdin.on("finish", () => this.mark({ kind: "stdin-finish" }));
    this.child.stdin.on("close", () => this.mark({ kind: "stdin-close" }));
    this.child.stdout.on("end", () => this.mark({ kind: "stdout-end" }));
    this.child.stderr.on("end", () => this.mark({ kind: "stderr-end" }));
    this.child.stderr.on("data", (bytes) => {
      this.stderr.push(Buffer.from(bytes));
      this.mark({ kind: "stderr-bytes", bytes: bytes.length, sha256: sha256(bytes) });
    });
    this.child.stdout.on("data", (bytes) => {
      this.output.push(Buffer.from(bytes));
      this.mark({ kind: "stdout-bytes", bytes: bytes.length, sha256: sha256(bytes) });
      this.pending = Buffer.concat([this.pending, bytes]);
      try {
        this.parse();
      } catch (error) {
        this.failure = String(error);
        this.mark({ kind: "framing-error", error: String(error) });
      }
      this.wake();
    });
  }
  mark(event) {
    this.events.push({
      index: this.events.length,
      auditorMonotonicNs: process.hrtime.bigint().toString(),
      ...event,
    });
  }
  wake() {
    for (const callback of this.waiters) callback();
  }
  parse() {
    while (true) {
      const frame = takeFrame(this.pending);
      if (!frame) return;
      this.pending = this.pending.subarray(frame.bytes);
      this.mark({ direction: "server", ...frame });
      const packet = frame.packet;
      if (packet.method && packet.id !== undefined) {
        assert.ok(
          [
            "client/registerCapability",
            "client/unregisterCapability",
            "workspace/diagnostic/refresh",
          ].includes(packet.method),
          `unqualified server request: ${packet.method}`,
        );
        this.send({ jsonrpc: "2.0", id: packet.id, result: null });
      }
    }
  }
  send(packet) {
    const body = Buffer.from(JSON.stringify(packet));
    const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
    this.input.push(frame);
    this.mark({ direction: "client", packet, bytes: frame.length, bodySha256: sha256(body) });
    this.child.stdin.write(frame);
  }
  notify(method, params) {
    this.send({ jsonrpc: "2.0", method, ...(params === undefined ? {} : { params }) });
  }
  async requestExact(packet) {
    assert.equal(packet.id, this.nextId + 1);
    assert.equal(packet.jsonrpc, "2.0");
    assert.equal(typeof packet.method, "string");
    this.nextId = packet.id;
    this.send(packet);
    return this.wait((response) => response.id === packet.id && !response.method, packet.method);
  }
  async request(method, params) {
    return this.requestExact({
      jsonrpc: "2.0",
      id: this.nextId + 1,
      method,
      ...(params === undefined ? {} : { params }),
    });
  }
  async wait(predicate, operation, from = 0) {
    const deadline = performance.now() + 30_000;
    while (true) {
      const found = this.events
        .slice(from)
        .find((event) => event.direction === "server" && predicate(event.packet));
      if (found) return found;
      assert.equal(this.failure, null, this.failure ?? operation);
      assert.equal(this.status, null, `SDK exited before ${operation}`);
      const remaining = deadline - performance.now();
      assert.ok(remaining > 0, `safety watchdog expired: ${operation}`);
      await new Promise((resolve, reject) => {
        const callback = () => {
          clearTimeout(timer);
          this.waiters.delete(callback);
          resolve();
        };
        const timer = setTimeout(() => {
          this.waiters.delete(callback);
          reject(new Error(`safety watchdog expired: ${operation}`));
        }, remaining);
        this.waiters.add(callback);
      });
    }
  }
  async end(reason) {
    this.mark({ kind: "stdin-end-called", reason });
    this.child.stdin.end();
    let timer;
    try {
      await Promise.race([
        this.closed,
        new Promise((_, reject) => {
          timer = setTimeout(() => reject(new Error("SDK close safety watchdog expired")), 30_000);
        }),
      ]);
    } finally {
      clearTimeout(timer);
      if (this.status === null) {
        this.forcedKill = true;
        this.mark({ kind: "forced-kill", signal: "SIGKILL" });
        this.child.kill("SIGKILL");
        await this.closed;
      }
      this.save();
    }
  }
  save() {
    const streams = {};
    for (const [name, chunks] of Object.entries({
      stdin: this.input,
      stdout: this.output,
      stderr: this.stderr,
    })) {
      const bytes = Buffer.concat(chunks),
        destination = path.join(this.directory, `${name}.bin`);
      fs.writeFileSync(destination, bytes);
      const record = { path: destination, bytes: bytes.length, sha256: sha256(bytes) };
      if (name !== "stderr") {
        try {
          record.frames = completeFrames(bytes);
          record.framing = "complete";
        } catch (error) {
          record.framing = "REFUSED";
          record.error = String(error);
        }
      }
      streams[name] = record;
    }
    const observation = {
      role: "direct SDK child of Node auditor",
      executable: this.executable,
      argv: this.argv,
      pid: this.child.pid,
      workspace: this.workspace,
      status: this.status,
      failure: this.failure,
      forcedKill: this.forcedKill ?? false,
      pendingBytes: this.pending.length,
      streams,
      events: this.events,
    };
    fs.writeFileSync(
      path.join(this.directory, "session.json"),
      JSON.stringify(observation, null, 2) + "\n",
    );
    return observation;
  }
}
