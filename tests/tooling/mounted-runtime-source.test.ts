import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { test } from "node:test";
import { collectRuntimeProducer, runtimeSourceCache } from "./support/mounted-runtime-source.ts";

test("one generated-code promise per mode preserves identity and never retries failure", async () => {
  const calls: boolean[] = [];
  const failure = new Error("original producer failure");
  const source = runtimeSourceCache(async (production) => {
    calls.push(production);
    if (production) throw failure;
    return "export const value = 1;";
  });
  const dev = source(false);
  const prod = source(true);
  assert.equal(source(false), dev, "concurrent development requests share their attempt");
  assert.equal(source(true), prod, "concurrent production requests share their attempt");
  assert.equal(await dev, "export const value = 1;");
  await assert.rejects(prod, (error) => error === failure);
  assert.equal(source(true), prod, "failed producer is retained, with no silent retry");
  await assert.rejects(source(true), (error) => error === failure);
  assert.equal(source(false), dev, "successful generated code is retained");
  assert.deepEqual(calls, [false, true]);
  assert.throws(() => source(undefined as unknown as boolean), assert.AssertionError);
});

function run(stdout: Buffer, stderr: Buffer, end = "") {
  const child = spawn(
    process.execPath,
    [
      "-e",
      `require('node:fs').writeSync(1, Buffer.from(process.argv[1], 'hex'));
       require('node:fs').writeSync(2, Buffer.from(process.argv[2], 'hex')); ${end}`,
      stdout.toString("hex"),
      stderr.toString("hex"),
    ],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  const result = collectRuntimeProducer(child);
  child.stdin.end();
  return result;
}

test("producer acceptance requires the complete canonical packet and strict natural exit", async () => {
  const packet = Buffer.from('{"code":"export const value = 1;"}\n');
  assert.equal(await run(packet, Buffer.alloc(0)), "export const value = 1;");
  const failures: [Buffer, Buffer, string][] = [
    [packet, Buffer.alloc(0), "process.exit(1)"],
    [packet, Buffer.from([255, 13, 27]), ""],
    [Buffer.from([254, 0, 10, 97]), Buffer.alloc(0), ""],
    [Buffer.from('{"code":"x","ignored":true}\n'), Buffer.alloc(0), ""],
    [Buffer.from('{"code":""}\n'), Buffer.alloc(0), ""],
    [Buffer.concat([packet, Buffer.from("\n")]), Buffer.alloc(0), ""],
    [Buffer.alloc(0), Buffer.alloc(0), ""],
  ];
  if (process.platform !== "win32")
    failures.push([packet, Buffer.alloc(0), "process.kill(process.pid, 'SIGTERM')"]);
  for (const [stdout, stderr, end] of failures) {
    await assert.rejects(run(stdout, stderr, end), (error) => {
      assert.ok(error instanceof Error);
      const [title, json] = error.message.split("\n");
      assert.equal(title, "mounted runtime producer failed");
      const evidence = JSON.parse(json);
      assert.deepEqual(evidence.stdoutBytes, [...stdout]);
      assert.deepEqual(evidence.stderrBytes, [...stderr]);
      assert.equal(evidence.stdout, stdout.toString("utf8"));
      assert.equal(evidence.stderr, stderr.toString("utf8"));
      assert.equal(evidence.exitCode, end === "process.exit(1)" ? 1 : end ? null : 0);
      assert.equal(evidence.signal, end.includes("SIGTERM") ? "SIGTERM" : null);
      assert.ok(Number.isInteger(evidence.pid));
      assert.equal(evidence.spawnError, null);
      assert.equal(evidence.inputError, null);
      assert.ok(error.cause instanceof Error);
      return true;
    });
  }
  const missing = spawn(`${process.execPath}.missing-${randomUUID()}`, [], {
    stdio: ["pipe", "pipe", "pipe"],
  });
  const missingResult = collectRuntimeProducer(missing);
  missing.stdin.end();
  await assert.rejects(missingResult, (error) => {
    assert.ok(error instanceof Error);
    const [title, json] = error.message.split("\n");
    assert.equal(title, "mounted runtime producer failed");
    const evidence = JSON.parse(json);
    assert.equal(evidence.pid, null);
    assert.equal(typeof evidence.exitCode, "number");
    assert.ok(evidence.exitCode < 0, "ENOENT must preserve its actual failed spawn status");
    assert.equal(evidence.signal, null);
    assert.match(evidence.spawnError, /ENOENT/u);
    assert.equal(evidence.inputError, null);
    assert.deepEqual(evidence.stdoutBytes, []);
    assert.deepEqual(evidence.stderrBytes, []);
    assert.equal(evidence.stdout, "");
    assert.equal(evidence.stderr, "");
    assert.ok(error.cause instanceof Error);
    return true;
  });
});
