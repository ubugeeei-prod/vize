import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { test } from "node:test";

const actionDir = new URL("../../.github/actions/setup-moonbit/", import.meta.url);
const source = fs.readFileSync(new URL("install-moonbit.mjs", actionDir), "utf8");
const transport =
  "Error: update failed\n\nCaused by:\n  0: failed to clone registry index\n  1: non-zero exit code: exit status: 128\n     git stderr:\n     fatal: unable to access 'https://mooncakes.io/git/index/': The requested URL returned error: 504\n";
const payload = Buffer.from("stdin 日本\r\n\0retained");
const writeAll =
  "function writeAll(fd, bytes) { for (let offset = 0; offset < bytes.length;) offset += fs.writeSync(fd, bytes, offset, bytes.length - offset); }";

function command(file, body) {
  fs.writeFileSync(file, `#!/usr/bin/env node\nimport fs from "node:fs";\n${writeAll}\n${body}\n`);
  fs.chmodSync(file, 0o755);
}

async function child(file, env, { input = Buffer.alloc(0), slowStderr = false, onStderr } = {}) {
  const processChild = spawn(process.execPath, [file], { env, stdio: "pipe" });
  const stdout = [];
  const stderr = [];
  let paused = false;
  processChild.stdout.on("data", (bytes) => stdout.push(bytes));
  processChild.stderr.on("data", (bytes) => {
    stderr.push(bytes);
    onStderr?.(bytes);
    if (slowStderr && !paused) {
      paused = true;
      processChild.stderr.pause();
      setTimeout(() => processChild.stderr.resume(), 100);
    }
  });
  processChild.stdin.end(input);
  const result = await new Promise((resolve, reject) => {
    processChild.once("error", reject);
    processChild.once("close", (status, signal) => resolve({ status, signal }));
  });
  return {
    ...result,
    pid: processChild.pid,
    stdout: Buffer.concat(stdout),
    stderr: Buffer.concat(stderr),
  };
}

function rawStderr(bytes) {
  return Buffer.from(
    bytes
      .toString("latin1")
      .replace(
        /^MoonBit registry update transport failed \(attempt [12]\/3, exit \d+\); retrying in [12]000ms\n/gm,
        "",
      ),
    "latin1",
  );
}

async function helper(attempts, { live = false } = {}) {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "moon streams 日本-"));
  try {
    const log = path.join(temp, "calls.jsonl");
    const state = path.join(temp, "state");
    const ack = path.join(temp, "live-ack");
    const fake = path.join(temp, "moon.mjs");
    const harness = path.join(temp, "harness.mjs");
    const env = {
      ...process.env,
      MOON_HOME: path.join(temp, "moonbit"),
      PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
    };
    attempts = attempts.map((attempt) => {
      const bytes = Buffer.from(attempt.stderrHex ?? "", "hex");
      const token = Buffer.from("__CONTROLLED_REGISTRY_INDEX__");
      const index = bytes.indexOf(token);
      const stderr =
        index < 0
          ? bytes
          : Buffer.concat([
              bytes.subarray(0, index),
              Buffer.from(path.join(env.MOON_HOME, "registry", "index")),
              bytes.subarray(index + token.length),
            ]);
      return { ...attempt, stderrHex: stderr.toString("hex") };
    });
    command(
      fake,
      `const attempts = ${JSON.stringify(attempts)};
const state = ${JSON.stringify(state)};
const index = fs.existsSync(state) ? Number(fs.readFileSync(state, "utf8")) : 0;
fs.writeFileSync(state, String(index + 1));
fs.appendFileSync(${JSON.stringify(log)}, JSON.stringify({ pid: process.pid, argv: process.argv.slice(2), execPath: process.execPath, moonHome: process.env.MOON_HOME, path: process.env.PATH, stdinHex: fs.readFileSync(0).toString("hex"), started: Date.now() }) + "\\n");
const current = attempts[index] ?? { status: 90 };
writeAll(1, Buffer.from(current.stdoutHex ?? "", "hex"));
writeAll(2, Buffer.from(current.stderrHex ?? "", "hex"));
if (current.waitForAck) {
  const ack = ${JSON.stringify(ack)};
  const until = Date.now() + 3000;
  while (!fs.existsSync(ack) && Date.now() < until) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
  if (!fs.existsSync(ack)) process.exit(88);
}
process.exit(current.status);`,
    );
    const start = source.indexOf("\nasync function updateRegistry(");
    const end = source.indexOf("\n}\n", start);
    assert.ok(start >= 0 && end > start);
    fs.writeFileSync(
      harness,
      `import { spawn } from "node:child_process";\nimport path from "node:path";\n${source.slice(start, end + 3)}\nawait updateRegistry(${JSON.stringify(process.execPath)}, ${JSON.stringify([fake, "update"])}, ${JSON.stringify(env)});\n`,
    );
    const result = await child(harness, env, {
      input: payload,
      slowStderr: true,
      onStderr: live ? () => fs.writeFileSync(ack, "received before inner child exit") : undefined,
    });
    const calls = fs.readFileSync(log, "utf8").trim().split("\n").map(JSON.parse);
    assert.ok(result.pid > 0);
    assert.equal(new Set(calls.map((call) => call.pid)).size, calls.length);
    for (const [index, call] of calls.entries()) {
      assert.ok(call.pid > 0 && call.pid !== result.pid);
      assert.deepEqual(call.argv, ["update"]);
      assert.equal(call.execPath, process.execPath);
      assert.equal(call.moonHome, env.MOON_HOME);
      assert.equal(call.path, env.PATH);
      assert.equal(call.stdinHex, index === 0 ? payload.toString("hex") : "");
    }
    assert.deepEqual(
      result.stdout,
      Buffer.concat(attempts.map((attempt) => Buffer.from(attempt.stdoutHex ?? "", "hex"))),
    );
    const expectedStderr = Buffer.concat(
      attempts.map((attempt) => Buffer.from(attempt.stderrHex ?? "", "hex")),
    );
    assert.ok(
      rawStderr(result.stderr).equals(expectedStderr),
      `whole stderr must match: actual ${rawStderr(result.stderr).length}, expected ${expectedStderr.length}`,
    );
    return { ...result, calls };
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}

const attempt = (status, tag, large = false) => ({
  status,
  stdoutHex: Buffer.from(`${tag}: stdout 日本\r\n\0`).toString("hex"),
  stderrHex: Buffer.concat([
    Buffer.from(`${tag}: stderr\r\n`),
    large ? Buffer.alloc(2 * 1024 * 1024, 255) : Buffer.from([0, 255]),
    Buffer.from(`\n${transport}`),
  ]).toString("hex"),
});

void test("terminal registry failure drains complete large stderr to a slow pipe reader", async () => {
  const terminal = attempt(255, "terminal", true);
  terminal.stderrHex += Buffer.from("fatal: Authentication failed\n").toString("hex");
  const result = await helper([terminal]);
  assert.equal(result.status, 255);
  assert.equal(result.calls.length, 1);
});

void test("successful registry retry preserves exact ordered streams, inner PIDs and inherited stdin", async () => {
  const result = await helper([attempt(255, "first"), attempt(0, "second")]);
  assert.equal(result.status, 0);
  assert.equal(result.calls.length, 2);
  assert.ok(result.calls[1].started - result.calls[0].started >= 1000);
});

const rpc = (http = 504) =>
  `Error: update failed\n\nCaused by:\n  0: failed to clone registry index\n  1: non-zero exit code: exit status: 128\n     git stderr:\n     Cloning into '__CONTROLLED_REGISTRY_INDEX__'...\n     error: RPC failed; HTTP ${http} curl 22 The requested URL returned error: ${http}\n     fatal: expected 'packfile'\n`;

for (const http of [502, 503, 504]) {
  void test(`source-bound registry RPC HTTP ${http} recovers with outer Moon exit255`, async () => {
    const result = await helper([
      { status: 255, stderrHex: Buffer.from(rpc(http)).toString("hex") },
      { status: 0, stdoutHex: Buffer.from("rpc-ready\n").toString("hex") },
    ]);
    assert.equal(result.status, 0);
    assert.equal(result.calls.length, 2);
    assert.ok(result.calls[1].started - result.calls[0].started >= 1000);
  });
}

for (const [name, status, stderr] of [
  ["RPC HTTP500", 255, rpc(500)],
  [
    "other registry destination",
    255,
    rpc().replace("__CONTROLLED_REGISTRY_INDEX__", "/other/registry/index"),
  ],
  ["other curl code", 255, rpc().replace("curl 22", "curl 7")],
  ["mismatched HTTP status", 255, rpc(502).replace("returned error: 502", "returned error: 504")],
  ["other Moon exit", 19, rpc()],
  [
    "missing packfile failure",
    255,
    rpc().replace("fatal: expected 'packfile'", "fatal: unknown transport error"),
  ],
  [
    "generic failure without HTTP",
    255,
    rpc().replace(/error: RPC failed;[^\n]+/, "error: generic transport failed"),
  ],
  [
    "other Git operation",
    255,
    rpc().replace("failed to clone registry index", "failed to clone project"),
  ],
  ["mixed authentication failure", 255, `${rpc()}fatal: Authentication failed\n`],
  ["mixed compiler failure", 255, `${rpc()}error: compiler failure\n`],
  [
    "explicit other Git URL",
    255,
    `${rpc()}fatal: unable to access 'https://example.test/git/index/': The requested URL returned error: 504\n`,
  ],
]) {
  void test(`${name} is not admitted as the owned registry RPC profile`, async () => {
    const result = await helper([{ status, stderrHex: Buffer.from(stderr).toString("hex") }]);
    assert.equal(result.status, status);
    assert.equal(result.calls.length, 1);
    assert.doesNotMatch(result.stderr.toString(), /retrying/);
  });
}

void test("stderr is copied live while the inner Moon child waits for a reader acknowledgement", async () => {
  const result = await helper(
    [
      {
        status: 0,
        waitForAck: true,
        stderrHex: Buffer.from("live-registry-stderr\n").toString("hex"),
      },
    ],
    { live: true },
  );
  assert.equal(result.status, 0);
  assert.equal(result.calls.length, 1);
});
