import assert from "node:assert/strict";
import { test } from "node:test";
import { collectLinterAttempts } from "../differential/linter-process.ts";
import { runNative } from "../differential/linter-native.ts";
import { sha256 } from "../differential/harness.mjs";

const input = Buffer.from('{"entry":"script"}');
const empty = Buffer.alloc(0);
function raw(stdout: Buffer, stderr: Buffer, exitStatus: number | null, error: string | null) {
  return {
    stdoutBase64: stdout.toString("base64"),
    stdoutSha256: sha256(stdout),
    stderrBase64: stderr.toString("base64"),
    stderrSha256: sha256(stderr),
    exitStatus,
    signal: null,
    processError: error,
  };
}

void test("legacy/native collection retains complete returned failures and the later raw attempt", () => {
  const calls: any[] = [];
  const returned = [
    {
      stdout: Buffer.from("failed\n"),
      stderr: Buffer.from("original failure\n"),
      status: 1,
      signal: null,
    },
    { stdout: Buffer.from("later whole output\n"), stderr: empty, status: 0, signal: null },
  ];
  const invoke: any = (...args: any[]) => {
    calls.push(args);
    return returned[calls.length - 1];
  };
  assert.deepEqual(collectLinterAttempts("/original/observer", ["--current-api"], input, invoke), [
    raw(returned[0].stdout, returned[0].stderr, 1, null),
    raw(returned[1].stdout, empty, 0, null),
  ]);
  assert.deepEqual(calls, [
    [
      "/original/observer",
      ["--current-api"],
      { input, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 },
    ],
    [
      "/original/observer",
      ["--current-api"],
      { input, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 },
    ],
  ]);
});

void test("a thrown-first native invocation keeps the second whole stream and cannot grant credit", () => {
  const calls: any[] = [];
  const later = Buffer.from("later whole stream\n");
  const invoke: any = (...args: any[]) => {
    calls.push(args);
    if (calls.length === 1) throw new Error("first invocation failed");
    return { stdout: later, stderr: empty, status: 0, signal: null };
  };
  const fixture = {
    argv: ["--current-api"],
    input,
    expected: Buffer.from("original whole output\n"),
  };
  const result = runNative("/original/observer", fixture, {}, invoke);
  const { error, ...whole } = result;
  assert.equal(typeof error, "string");
  assert.match(error, /first invocation failed/);
  assert.deepEqual(whole, {
    state: "failed",
    argv: ["--native-current-api"],
    inputSha256: sha256(input),
    attempts: [raw(empty, empty, null, "first invocation failed"), raw(later, empty, 0, null)],
  });
  assert.deepEqual(calls, [
    [
      "/original/observer",
      ["--native-current-api"],
      { input, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 },
    ],
    [
      "/original/observer",
      ["--native-current-api"],
      { input, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 },
    ],
  ]);
});
