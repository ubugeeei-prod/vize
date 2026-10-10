import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const installer = fileURLToPath(
  new URL("../../.github/actions/setup-moonbit/install-moonbit.mjs", import.meta.url),
);
const source = fs.readFileSync(installer, "utf8");

// Execute the exact installer function in a child process. A fake Moon child
// records its argv and environment; neither toolchain installation nor network
// access is needed to exercise its real exit statuses and raw output streams.
function installerFunction(name) {
  const marker = name === "run" ? "\nfunction run(" : "\nasync function updateRegistry(";
  const start = source.indexOf(marker);
  if (start === -1 && name !== "run") {
    // Before the fix, execute the original one-shot body under the same async
    // harness so recovery assertions fail on the actual original exit status.
    return installerFunction("run").replace("function run(", "async function updateRegistry(");
  }
  assert.notEqual(start, -1, `installer function ${name} is required`);
  const end = source.indexOf("\n}\n", start);
  assert.notEqual(end, -1);
  return source.slice(start, end + 3);
}

function transportFailure(http = 504, url = "https://mooncakes.io/git/index/", gitStatus = 128) {
  return `Error: update failed\n\nCaused by:\n    0: failed to clone registry index\n    1: non-zero exit code: exit status: ${gitStatus}\n       git stderr:\n       Cloning into '/runner temp/moonbit/registry/index'...\n       fatal: unable to access '${url}': The requested URL returned error: ${http}\n`;
}

function exercise(attempts, { original = false, missing = false } = {}) {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "moon registry 日本-"));
  try {
    const moonHome = path.join(temp, "moonbit");
    const log = path.join(temp, "argv.jsonl");
    const state = path.join(temp, "attempt-count");
    const fakeMoon = path.join(temp, "fake-moon.mjs");
    const harness = path.join(temp, "installer-harness.mjs");
    const childEnv = {
      ...process.env,
      MOON_HOME: moonHome,
      PATH: `${path.join(moonHome, "bin")}${path.delimiter}${process.env.PATH ?? ""}`,
    };
    fs.writeFileSync(
      fakeMoon,
      `import fs from "node:fs";\nconst attempts = ${JSON.stringify(attempts)};\n` +
        `const state = ${JSON.stringify(state)};\nconst log = ${JSON.stringify(log)};\n` +
        `const index = fs.existsSync(state) ? Number(fs.readFileSync(state, "utf8")) : 0;\n` +
        `fs.writeFileSync(state, String(index + 1));\n` +
        `fs.appendFileSync(log, JSON.stringify({ argv: process.argv.slice(2), execPath: process.execPath, moonHome: process.env.MOON_HOME, path: process.env.PATH }) + "\\n");\n` +
        `const attempt = attempts[index] ?? { status: 90, stderr: "unexpected extra attempt" };\n` +
        `process.stdout.write(Buffer.from(attempt.stdoutHex ?? "", "hex"));\n` +
        `process.stderr.write(Buffer.from(attempt.stderrHex ?? Buffer.from(attempt.stderr ?? "").toString("hex"), "hex"));\n` +
        `if (attempt.signal) process.kill(process.pid, attempt.signal);\n` +
        `else process.exit(attempt.status);\n`,
    );
    const name = original ? "run" : "updateRegistry";
    const command = missing ? path.join(temp, "missing-moon") : process.execPath;
    const args = missing ? ["update"] : [fakeMoon, "update"];
    fs.writeFileSync(
      harness,
      `import { spawn, spawnSync } from "node:child_process";\n` +
        installerFunction(name) +
        `\n${original ? "" : "await "}${name}(${JSON.stringify(command)}, ${JSON.stringify(args)}, ${JSON.stringify(childEnv)});\n`,
    );
    const result = spawnSync(process.execPath, [harness]);
    const calls = fs.existsSync(log)
      ? fs.readFileSync(log, "utf8").trim().split("\n").map(JSON.parse)
      : [];
    for (const call of calls) {
      assert.deepEqual(call.argv, ["update"]);
      assert.equal(call.execPath, process.execPath);
      assert.equal(call.moonHome, moonHome);
      assert.equal(call.path, childEnv.PATH);
    }
    return { ...result, calls };
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}

const failure = (http = 504, status = 255) => ({ status, stderr: transportFailure(http) });
const success = { status: 0, stdoutHex: Buffer.from("registry-ready\n").toString("hex") };

function assertRaw(result, attempts) {
  for (const attempt of attempts) {
    const stdout = Buffer.from(attempt.stdoutHex ?? "", "hex");
    const stderr = Buffer.from(
      attempt.stderrHex ?? Buffer.from(attempt.stderr ?? "").toString("hex"),
      "hex",
    );
    assert.ok(result.stdout.includes(stdout), "whole child stdout must survive");
    assert.ok(result.stderr.includes(stderr), "whole child stderr must survive");
  }
}

void test("original one-shot installer exits before a recoverable second attempt", () => {
  const attempts = [failure(), success];
  const result = exercise(attempts, { original: true });
  assert.equal(result.status, 255);
  assert.equal(result.calls.length, 1);
  assertRaw(result, attempts.slice(0, 1));
  assert.equal(result.stdout.includes(Buffer.from("registry-ready")), false);
});

for (const http of [502, 503, 504]) {
  void test(`registry HTTP ${http} recovers with unchanged command arguments and environment`, () => {
    const attempts = [failure(http), success];
    const result = exercise(attempts);
    assert.equal(result.status, 0);
    assert.equal(result.calls.length, 2);
    assertRaw(result, attempts);
    assert.match(result.stderr.toString(), /attempt 1\/3.*1000ms/);
  });
}

void test("three transient failures preserve raw bytes and the last exit status", () => {
  const attempts = [502, 503, 504].map((http, index) => ({
    status: [255, 128, 19][index],
    stdoutHex: Buffer.from([0, 255, index, 13, 10]).toString("hex"),
    stderrHex: Buffer.concat([
      Buffer.from(`attempt-${index}: 日本\r\n`),
      Buffer.from([255, index, 13, 10]),
      Buffer.from(transportFailure(http)),
    ]).toString("hex"),
  }));
  const result = exercise(attempts);
  assert.equal(result.status, 19);
  assert.equal(result.calls.length, 3);
  assertRaw(result, attempts);
  assert.match(result.stderr.toString(), /attempt 1\/3.*1000ms/);
  assert.match(result.stderr.toString(), /attempt 2\/3.*2000ms/);
  assert.doesNotMatch(result.stderr.toString(), /attempt 3\/3.*retrying/);
});

for (const [name, stderr] of [
  ["authentication", `${transportFailure()}fatal: Authentication failed\n`],
  ["authorization", `${transportFailure()}fatal: Permission denied\n`],
  ["validation", `${transportFailure()}error: validation failed\n`],
  ["compiler", `${transportFailure()}error: compiler failure\n`],
  ["installer hash", `${transportFailure()}installer hash mismatch\n`],
  ["compiler version", `${transportFailure()}version mismatch\n`],
  ["HTTP 401", transportFailure(401)],
  ["HTTP 403", transportFailure(403)],
  ["HTTP 408", transportFailure(408)],
  ["HTTP 429", transportFailure(429)],
  ["HTTP 500", transportFailure(500)],
  ["another registry", transportFailure(504, "https://example.test/git/index/")],
  ["another Git status", transportFailure(504, "https://mooncakes.io/git/index/", 1)],
  [
    "unrelated command",
    transportFailure().replace("failed to clone registry index", "failed to compile package"),
  ],
]) {
  void test(`${name} failure is terminal on its first attempt`, () => {
    const attempts = [{ status: 23, stderr }, success];
    const result = exercise(attempts);
    assert.equal(result.status, 23);
    assert.equal(result.calls.length, 1);
    assertRaw(result, attempts.slice(0, 1));
    assert.doesNotMatch(result.stderr.toString(), /retrying/);
  });
}

void test("a registry fetch transport failure uses the same bounded retry", () => {
  const attempts = [
    { status: 255, stderr: transportFailure(503).replace("failed to clone", "failed to fetch") },
    success,
  ];
  const result = exercise(attempts);
  assert.equal(result.status, 0);
  assert.equal(result.calls.length, 2);
  assertRaw(result, attempts);
});

void test("successful first attempt does not retry diagnostic-looking stderr", () => {
  const attempts = [{ ...success, stderr: transportFailure() }];
  const result = exercise(attempts);
  assert.equal(result.status, 0);
  assert.equal(result.calls.length, 1);
  assertRaw(result, attempts);
  assert.doesNotMatch(result.stderr.toString(), /retrying/);
});

void test(
  "a signaled child preserves the original terminal fallback",
  { skip: process.platform === "win32" },
  () => {
    const result = exercise([{ signal: "SIGTERM", stderr: transportFailure() }, success]);
    assert.equal(result.status, 1);
    assert.equal(result.calls.length, 1);
    assert.doesNotMatch(result.stderr.toString(), /retrying/);
  },
);

void test("a missing executable fails without retry", () => {
  const result = exercise([], { missing: true });
  assert.equal(result.status, 1);
  assert.equal(result.calls.length, 0);
  assert.doesNotMatch(result.stderr.toString(), /retrying/);
});

void test("only the existing cold-install registry update uses retry", () => {
  assert.equal(source.match(/await updateRegistry\(moonExe, \["update"\], \{/g)?.length, 1);
  assert.ok(
    source.indexOf("await updateRegistry(moonExe") >
      source.indexOf("if (installed !== moonbitVersion)"),
  );
  assert.ok(source.indexOf("smokeTestMoon();") > source.indexOf("await updateRegistry(moonExe"));
});
