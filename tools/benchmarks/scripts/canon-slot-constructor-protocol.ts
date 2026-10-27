/** Untimed direct mapper transaction: stdin END and every raw frame are retained. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  readSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";

const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function image(path: string) {
  const fd = openSync(path, "r"),
    buffer = Buffer.allocUnsafe(1024 * 1024);
  const hash = createHash("sha256");
  try {
    for (;;) {
      const bytes = readSync(fd, buffer, 0, buffer.length, null);
      if (bytes === 0) break;
      hash.update(buffer.subarray(0, bytes));
    }
  } finally {
    closeSync(fd);
  }
  return { path: resolve(path), bytes: statSync(path).size, sha256: hash.digest("hex") };
}
export function slotRequests(project: string, source: string, config: unknown) {
  assert.ok(record(config) && record(config.compilerOptions), "parsed compilerOptions required");
  const params: [string, Record<string, unknown>][] = [
    ["initialize", { positionEncodings: ["utf-8", "utf-16"] }],
    [
      "openProject",
      {
        configFileName: join(project, "tsconfig.mapper.json"),
        projectHandle: "slot-proof",
        options: null,
        compilerOptions: config.compilerOptions,
      },
    ],
    [
      "transform",
      { fileName: join(project, "src/Tiny.vue"), content: source, projectHandle: "slot-proof" },
    ],
    ["closeProject", { projectHandle: "slot-proof" }],
  ];
  return Buffer.concat(
    params.map(([method, value], index) => {
      const payload = Buffer.from(
        JSON.stringify({ jsonrpc: "2.0", id: index + 1, method, params: value }),
        "utf8",
      );
      return Buffer.concat([
        Buffer.from("Content-Length: " + payload.length + "\r\n\r\n"),
        payload,
      ]);
    }),
  );
}
export function slotResponses(bytes: Buffer) {
  const responses: Record<string, unknown>[] = [];
  let position = 0;
  const utf8 = new TextDecoder("utf-8", { fatal: true });
  while (position < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", position, "ascii");
    assert.ok(end >= position, "incomplete response header");
    const header = bytes.subarray(position, end).toString("ascii");
    const match = /^Content-Length: ([0-9]+)$/u.exec(header);
    assert.ok(match?.[1], "unexpected response header");
    const length = Number(match[1]),
      start = end + 4;
    assert.ok(
      Number.isSafeInteger(length) && length > 0 && length <= bytes.length - start,
      "response byte length exceeds complete payload",
    );
    const value: unknown = JSON.parse(utf8.decode(bytes.subarray(start, start + length)));
    assert.ok(record(value), "response must be an object");
    assert.equal(value.jsonrpc, "2.0");
    assert.equal(value.id, responses.length + 1, "all four IDs must occur once in order");
    assert.ok(
      Object.hasOwn(value, "result") && !Object.hasOwn(value, "error"),
      "RPC error or missing result",
    );
    responses.push(value);
    position = start + length;
  }
  assert.equal(responses.length, 4, "exactly four complete responses required");
  assert.equal(position, bytes.length, "trailing bytes");
  const transform = responses[2],
    close = responses[3];
  assert.ok(transform && record(transform.result), "transform result missing");
  assert.equal(typeof transform.result.text, "string", "virtual TS text missing");
  assert.ok(close);
  assert.equal(close.result, null);
  return { responses, result: transform.result };
}
export function captureSlotPacket(binary: string, project: string, source: string, out: string) {
  binary = resolve(binary);
  project = resolve(project);
  out = resolve(out);
  assert.ok(!existsSync(out), "fresh packet evidence path required");
  mkdirSync(out, { recursive: true });
  const configBytes = readFileSync(join(project, "tsconfig.mapper.json"));
  const config: unknown = JSON.parse(configBytes.toString("utf8"));
  const input = slotRequests(project, source, config),
    before = image(binary);
  writeFileSync(join(out, "stdin.raw"), input);
  writeFileSync(join(out, "config.bytes"), configBytes);
  const child = spawnSync(binary, ["content-mapper"], {
    cwd: project,
    input,
    encoding: "buffer",
    timeout: 60_000,
    maxBuffer: 32 * 1024 * 1024,
    killSignal: "SIGKILL",
  });
  const stdout = child.stdout ?? Buffer.alloc(0),
    stderr = child.stderr ?? Buffer.alloc(0);
  writeFileSync(join(out, "stdout.raw"), stdout);
  writeFileSync(join(out, "stderr.raw"), stderr);
  const after = image(binary);
  const rawStream = (name: string, bytes: Buffer) => ({
    path: join(out, name + ".raw"),
    bytes: bytes.length,
    sha256: digest(bytes),
  });
  const raw = {
    stdin: rawStream("stdin", input),
    stdout: rawStream("stdout", stdout),
    stderr: rawStream("stderr", stderr),
  };
  const receipt = {
    schemaVersion: 1,
    executable: before,
    executableAfter: after,
    argv: ["content-mapper"],
    cwd: project,
    config: { bytes: configBytes.length, sha256: digest(configBytes) },
    raw,
    stdinEndRequested: true,
    childExitObserved: child.status !== null,
    status: child.status,
    signal: child.signal,
    error: child.error?.message ?? null,
    captureQualified: false,
    timingUsable: false,
    allocationCounts: null,
    scope:
      "direct transaction with supplied stdin END; does not prove the SDK-hosted proxy's shutdown",
  };
  writeFileSync(join(out, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  assert.equal(child.error, undefined, child.error?.message);
  assert.equal(child.status, 0, stderr.toString("utf8"));
  assert.equal(child.signal, null);
  assert.equal(before.sha256, after.sha256);
  const packet = slotResponses(stdout);
  receipt.captureQualified = true;
  writeFileSync(join(out, "responses.json"), JSON.stringify(packet.responses, null, 2) + "\n");
  writeFileSync(join(out, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  return { ...receipt, result: packet.result, responses: packet.responses };
}
