import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import type { JsonRpcMessage } from "../../../../../tests/tooling/support/lsp/protocol.ts";

export type ResourceCorsaGatePaths = {
  root: string;
  proxyPath: string;
  actualCorsaPath: string;
};

export type GateProcessIdentity = {
  role: "proxy" | "native";
  transport: "lsp" | "other";
  pid: number;
  birth_ticks: number;
  executable: string;
};

export type NativeFrameEvidence = {
  bytes: number;
  sha256: string;
  base64: string;
  message: JsonRpcMessage;
};

export type NativeAdmissionReceipt = {
  token: number;
  proxy_entered_ns: string;
  harness_observed_entered_ns: string;
  actual_corsa_path: string;
  proxy_path: string;
  proxy_process: GateProcessIdentity;
  native_process: GateProcessIdentity;
  request: NativeFrameEvidence & { method: "textDocument/hover"; id: number };
  response: NativeFrameEvidence & { id: number };
};

export type GateReleaseReceipt = {
  token: number;
  proxy_released_ns: string;
  harness_observed_released_ns: string;
};

export type GateCleanupReceipt = {
  recorded_processes: GateProcessIdentity[];
  alive_after_grace_period: GateProcessIdentity[];
  signalled: Array<GateProcessIdentity & { signal: "SIGTERM" | "SIGKILL" }>;
  remaining_processes: GateProcessIdentity[];
  no_orphans: boolean;
  forced_cleanup_complete: boolean;
};

/** Distinguish graceful exit from a completed TERM/KILL fallback. */
export function gateCleanupStatus(
  aliveAfterGracePeriod: number,
  signalsSent: number,
  remainingProcesses: number,
) {
  for (const value of [aliveAfterGracePeriod, signalsSent, remainingProcesses]) {
    assert.ok(Number.isSafeInteger(value) && value >= 0, "cleanup counts must be non-negative");
  }
  return {
    no_orphans: aliveAfterGracePeriod === 0 && signalsSent === 0 && remainingProcesses === 0,
    forced_cleanup_complete: remainingProcesses === 0,
  };
}

const proxySource = String.raw`
const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");
const gate = JSON.parse(fs.readFileSync(path.join(__dirname, "gate.json"), "utf8"));
const args = process.argv.slice(2);
const framed = args.includes("--lsp");

function procIdentity(role, pid) {
  const stat = fs.readFileSync("/proc/" + pid + "/stat", "utf8");
  const fields = stat.slice(stat.lastIndexOf(") ") + 2).trim().split(/\s+/);
  return {
    role,
    transport: framed ? "lsp" : "other",
    pid,
    birth_ticks: Number(fields[19]),
    executable: fs.realpathSync("/proc/" + pid + "/exe"),
  };
}

function recordProcess(identity) {
  fs.appendFileSync(path.join(gate.root, "processes.jsonl"), JSON.stringify(identity) + "\n");
}

recordProcess(procIdentity("proxy", process.pid));
const child = spawn(gate.runtime, args, { stdio: framed ? ["pipe", "pipe", "inherit"] : "inherit" });
child.once("spawn", () => recordProcess(procIdentity("native", child.pid)));
child.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
child.on("exit", (code, signal) => {
  fs.appendFileSync(path.join(gate.root, "native-exits.jsonl"), JSON.stringify({ code, signal }) + "\n");
  if (signal) {
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  }
  else process.exitCode = code;
});
for (const signal of ["SIGTERM", "SIGINT"]) {
  process.on(signal, () => {
    child.kill(signal);
    process.exitCode = 1;
  });
}

function parseFrames(consume) {
  let buffered = Buffer.alloc(0);
  return (chunk) => {
    buffered = Buffer.concat([buffered, chunk]);
    while (true) {
      const boundary = buffered.indexOf("\r\n\r\n");
      if (boundary < 0) return;
      const match = /^Content-Length:\s*(\d+)$/im.exec(buffered.subarray(0, boundary).toString());
      if (!match) throw new Error("native frame has no Content-Length");
      const end = boundary + 4 + Number(match[1]);
      if (buffered.length < end) return;
      const frame = buffered.subarray(0, end);
      const message = JSON.parse(buffered.subarray(boundary + 4, end).toString("utf8"));
      buffered = buffered.subarray(end);
      consume(frame, message);
    }
  };
}

function tokenDirectory(token) {
  return path.join(gate.root, "tokens", String(token).padStart(2, "0"));
}

function armedToken() {
  try {
    const value = JSON.parse(fs.readFileSync(path.join(gate.root, "armed.json"), "utf8"));
    if (!Number.isSafeInteger(value.token) || value.token <= 0) throw new Error("invalid gate token");
    return value.token;
  } catch (error) {
    if (error && error.code === "ENOENT") return undefined;
    throw error;
  }
}

function writeFrame(frame) {
  return new Promise((resolve, reject) => {
    process.stdout.write(frame, (error) => error ? reject(error) : resolve());
  });
}

async function awaitRelease(directory) {
  const deadline = Date.now() + 120000;
  while (!fs.existsSync(path.join(directory, "release"))) {
    if (Date.now() >= deadline) throw new Error("resource native response gate was never released");
    await new Promise((resolve) => setTimeout(resolve, 2));
  }
}

if (framed) {
  let selected;
  let lastToken = 0;
  process.stdin.on("data", parseFrames((frame, message) => {
    if (message.method === "textDocument/hover" && selected === undefined) {
      const token = armedToken();
      if (token !== undefined && token > lastToken) {
        const directory = tokenDirectory(token);
        try {
          fs.writeFileSync(path.join(directory, "claimed"), "", { flag: "wx" });
          selected = { token, id: message.id, directory };
          fs.writeFileSync(path.join(directory, "request.bin"), frame);
          fs.writeFileSync(path.join(directory, "request.json"), JSON.stringify(message));
        } catch (error) {
          if (!error || error.code !== "EEXIST") throw error;
        }
      }
    }
    child.stdin.write(frame);
  }));
  process.stdin.on("end", () => child.stdin.end());
  let output = Promise.resolve();
  child.stdout.on("data", parseFrames((frame, message) => {
    output = output.then(async () => {
      if (selected !== undefined && message.id === selected.id) {
        const held = selected;
        fs.writeFileSync(path.join(held.directory, "response.bin"), frame);
        fs.writeFileSync(path.join(held.directory, "response.json"), JSON.stringify(message));
        fs.writeFileSync(
          path.join(held.directory, "entered.json"),
          JSON.stringify({ token: held.token, entered_ns: process.hrtime.bigint().toString() }),
        );
        await awaitRelease(held.directory);
        await writeFrame(frame);
        fs.writeFileSync(
          path.join(held.directory, "released.json"),
          JSON.stringify({ token: held.token, released_ns: process.hrtime.bigint().toString() }),
        );
        lastToken = held.token;
        selected = undefined;
        return;
      }
      await writeFrame(frame);
    }).catch((error) => {
      console.error(error);
      child.kill("SIGTERM");
      process.exitCode = 1;
    });
  }));
}
`;

export const resourceCorsaGateScript = (nodePath = process.execPath): string => {
  assert.ok(nodePath.length > 0 && !nodePath.includes("\n"), "gate Node path must fit one shebang");
  return `#!${nodePath}\n${proxySource}`;
};

const sha256 = (bytes: Buffer | string): string => createHash("sha256").update(bytes).digest("hex");

const assertRecord = (value: unknown, message: string): Record<string, unknown> => {
  assert.ok(typeof value === "object" && value !== null && !Array.isArray(value), message);
  return value as Record<string, unknown>;
};

/** Decode one complete Content-Length frame without normalizing its bytes. */
export function nativeFrameEvidence(frame: Buffer): NativeFrameEvidence {
  const boundary = frame.indexOf("\r\n\r\n");
  assert.ok(boundary >= 0, "native frame must contain a header boundary");
  const header = frame.subarray(0, boundary).toString("ascii");
  const match = /^Content-Length:\s*(\d+)$/im.exec(header);
  assert.ok(match, "native frame must declare Content-Length");
  const body = frame.subarray(boundary + 4);
  assert.equal(body.byteLength, Number(match[1]), "native frame body must be complete");
  const message: unknown = JSON.parse(body.toString("utf8"));
  assertRecord(message, "native frame body must be a JSON object");
  return {
    bytes: frame.byteLength,
    sha256: sha256(frame),
    base64: frame.toString("base64"),
    message: message as JsonRpcMessage,
  };
}

/** Install an executable, byte-preserving proxy around the real Corsa binary. */
export function installResourceCorsaGate(
  workspace: string,
  actualCorsaPath: string,
): ResourceCorsaGatePaths {
  assert.equal(process.platform, "linux", "the resource Corsa gate requires Linux /proc");
  const root = path.join(workspace, ".vize-corsa-gate");
  const proxyPath = path.join(root, "corsa-gate.cjs");
  fs.mkdirSync(path.join(root, "tokens"), { recursive: true });
  fs.writeFileSync(path.join(root, "package.json"), '{"type":"commonjs"}\n');
  fs.writeFileSync(
    path.join(root, "gate.json"),
    `${JSON.stringify({ runtime: fs.realpathSync(actualCorsaPath), root })}\n`,
  );
  fs.writeFileSync(proxyPath, resourceCorsaGateScript());
  fs.chmodSync(proxyPath, 0o755);
  return { root, proxyPath, actualCorsaPath: fs.realpathSync(actualCorsaPath) };
}

const tokenDirectory = (root: string, token: number): string =>
  path.join(root, "tokens", String(token).padStart(2, "0"));

const readJson = (file: string): unknown => JSON.parse(fs.readFileSync(file, "utf8"));

const processRecords = (root: string): GateProcessIdentity[] => {
  const file = path.join(root, "processes.jsonl");
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => {
      const row = assertRecord(JSON.parse(line), "gate process row must be an object");
      assert.ok(row.role === "proxy" || row.role === "native");
      assert.ok(row.transport === "lsp" || row.transport === "other");
      assert.ok(Number.isSafeInteger(row.pid) && Number(row.pid) > 0);
      assert.ok(Number.isSafeInteger(row.birth_ticks) && Number(row.birth_ticks) > 0);
      assert.equal(typeof row.executable, "string");
      return {
        role: row.role,
        transport: row.transport,
        pid: Number(row.pid),
        birth_ticks: Number(row.birth_ticks),
        executable: String(row.executable),
      };
    });
};

const currentIdentity = (identity: GateProcessIdentity): GateProcessIdentity | null => {
  try {
    const stat = fs.readFileSync(`/proc/${identity.pid}/stat`, "utf8");
    const fields = stat
      .slice(stat.lastIndexOf(") ") + 2)
      .trim()
      .split(/\s+/);
    return {
      role: identity.role,
      transport: identity.transport,
      pid: identity.pid,
      birth_ticks: Number(fields[19]),
      executable: fs.realpathSync(`/proc/${identity.pid}/exe`),
    };
  } catch {
    return null;
  }
};

const sameIdentity = (
  expected: GateProcessIdentity,
  current: GateProcessIdentity | null,
): boolean =>
  current !== null &&
  current.pid === expected.pid &&
  current.birth_ticks === expected.birth_ticks &&
  current.executable === expected.executable;

const blockingWait = (milliseconds: number): void => {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);
};

const waitUntilGone = (
  identities: readonly GateProcessIdentity[],
  milliseconds: number,
): GateProcessIdentity[] => {
  const deadline = Date.now() + milliseconds;
  let alive = identities.filter((identity) => sameIdentity(identity, currentIdentity(identity)));
  while (alive.length > 0 && Date.now() < deadline) {
    blockingWait(10);
    alive = identities.filter((identity) => sameIdentity(identity, currentIdentity(identity)));
  }
  return alive;
};

const signalRecordedProcess = (
  identity: GateProcessIdentity,
  signal: "SIGTERM" | "SIGKILL",
): boolean => {
  const current = currentIdentity(identity);
  if (current === null) return false;
  assert.ok(sameIdentity(identity, current), "refusing to signal a reused gate PID");
  try {
    process.kill(identity.pid, signal);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
    throw error;
  }
};

export class ResourceCorsaGate {
  readonly paths: ResourceCorsaGatePaths;
  #lastToken = 0;

  constructor(paths: ResourceCorsaGatePaths) {
    this.paths = paths;
  }

  arm(token: number): void {
    assert.equal(token, this.#lastToken + 1, "resource gate tokens must be contiguous");
    if (this.#lastToken > 0) {
      assert.ok(
        fs.existsSync(path.join(tokenDirectory(this.paths.root, this.#lastToken), "released.json")),
        `resource gate token ${this.#lastToken} must release before token ${token}`,
      );
    }
    const directory = tokenDirectory(this.paths.root, token);
    fs.mkdirSync(directory, { recursive: false });
    const temporary = path.join(this.paths.root, "armed.next.json");
    fs.writeFileSync(temporary, `${JSON.stringify({ token })}\n`);
    fs.renameSync(temporary, path.join(this.paths.root, "armed.json"));
    this.#lastToken = token;
  }

  release(token: number): void {
    const directory = tokenDirectory(this.paths.root, token);
    fs.writeFileSync(path.join(directory, "release"), "", { flag: "a" });
  }

  async waitEntered(token: number, timeoutMs = 120_000): Promise<NativeAdmissionReceipt> {
    const directory = tokenDirectory(this.paths.root, token);
    await this.#waitFor(path.join(directory, "entered.json"), timeoutMs);
    const entered = assertRecord(readJson(path.join(directory, "entered.json")), "entered receipt");
    assert.equal(entered.token, token);
    if (typeof entered.entered_ns !== "string") throw new TypeError("entered_ns must be a string");
    const enteredNs = entered.entered_ns;
    const request = nativeFrameEvidence(fs.readFileSync(path.join(directory, "request.bin")));
    const response = nativeFrameEvidence(fs.readFileSync(path.join(directory, "response.bin")));
    assert.equal(request.message.jsonrpc, "2.0");
    assert.equal(response.message.jsonrpc, "2.0");
    assert.equal(request.message.method, "textDocument/hover");
    assert.ok(Number.isSafeInteger(request.message.id) && Number(request.message.id) > 0);
    assert.equal(response.message.id, request.message.id);
    assert.ok(
      Object.hasOwn(response.message, "result") && response.message.error === undefined,
      "native hover must produce an actual successful response",
    );
    const identities = processRecords(this.paths.root);
    const live = identities.filter((identity) => sameIdentity(identity, currentIdentity(identity)));
    const proxies = live.filter(
      (identity) => identity.role === "proxy" && identity.transport === "lsp",
    );
    const natives = live.filter(
      (identity) => identity.role === "native" && identity.transport === "lsp",
    );
    assert.equal(proxies.length, 1, "gate must retain one live framed proxy identity");
    assert.equal(natives.length, 1, "gate must retain one live actual Corsa identity");
    const [proxy] = proxies;
    const [native] = natives;
    assert.equal(native.executable, this.paths.actualCorsaPath);
    return {
      token,
      proxy_entered_ns: enteredNs,
      harness_observed_entered_ns: process.hrtime.bigint().toString(),
      actual_corsa_path: this.paths.actualCorsaPath,
      proxy_path: this.paths.proxyPath,
      proxy_process: proxy,
      native_process: native,
      request: {
        ...request,
        method: "textDocument/hover",
        id: Number(request.message.id),
      },
      response: { ...response, id: Number(response.message.id) },
    };
  }

  async waitReleased(token: number, timeoutMs = 30_000): Promise<GateReleaseReceipt> {
    const file = path.join(tokenDirectory(this.paths.root, token), "released.json");
    await this.#waitFor(file, timeoutMs);
    const released = assertRecord(readJson(file), "released receipt");
    assert.equal(released.token, token);
    if (typeof released.released_ns !== "string")
      throw new TypeError("released_ns must be a string");
    const releasedNs = released.released_ns;
    return {
      token,
      proxy_released_ns: releasedNs,
      harness_observed_released_ns: process.hrtime.bigint().toString(),
    };
  }

  cleanup(): GateCleanupReceipt {
    if (this.#lastToken > 0) this.release(this.#lastToken);
    const recorded = processRecords(this.paths.root);
    const aliveAfterGrace = waitUntilGone(recorded, 1_000);
    const signalled: GateCleanupReceipt["signalled"] = [];
    for (const identity of aliveAfterGrace) {
      if (signalRecordedProcess(identity, "SIGTERM"))
        signalled.push({ ...identity, signal: "SIGTERM" });
    }
    const afterTerm = waitUntilGone(aliveAfterGrace, 1_000);
    for (const identity of afterTerm) {
      if (signalRecordedProcess(identity, "SIGKILL"))
        signalled.push({ ...identity, signal: "SIGKILL" });
    }
    const remaining = waitUntilGone(afterTerm, 1_000);
    const status = gateCleanupStatus(aliveAfterGrace.length, signalled.length, remaining.length);
    return {
      recorded_processes: recorded,
      alive_after_grace_period: aliveAfterGrace,
      signalled,
      remaining_processes: remaining,
      ...status,
    };
  }

  async #waitFor(file: string, timeoutMs: number): Promise<void> {
    const deadline = performance.now() + timeoutMs;
    while (!fs.existsSync(file)) {
      assert.ok(performance.now() < deadline, `timed out waiting for ${file}`);
      await sleep(2);
    }
  }
}
