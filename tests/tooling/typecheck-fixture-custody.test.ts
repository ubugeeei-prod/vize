import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { retainFixtureCommand } from "./support/typecheck-fixture-custody.ts";

test("fixture custody preserves binary failure streams and owned inputs before cleanup", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custody-test-"));
  try {
    const fixture = path.join(root, "fixture");
    fs.mkdirSync(fixture);
    const original = Buffer.from([0, 255, 128, 10]);
    fs.writeFileSync(path.join(fixture, "original.bin"), original);
    const args = [
      "-e",
      "process.stdout.write(Buffer.from([0,255,10]));process.stderr.write(Buffer.from([128,0]));process.exitCode=1;",
    ];
    const result = spawnSync(process.execPath, args, { encoding: "buffer" });
    const evidence = retainFixtureCommand(
      path.join(root, "retained"),
      { executable: process.execPath, args, cwd: root },
      result,
      { fixture },
    );
    fs.rmSync(fixture, { recursive: true });
    assert.equal(result.status, 1);
    assert.deepEqual(fs.readFileSync(path.join(evidence, "stdout.bin")), Buffer.from([0, 255, 10]));
    assert.deepEqual(fs.readFileSync(path.join(evidence, "stderr.bin")), Buffer.from([128, 0]));
    assert.deepEqual(fs.readFileSync(path.join(evidence, "fixture/original.bin")), original);
    const receipt = JSON.parse(fs.readFileSync(path.join(evidence, "command.json"), "utf8"));
    assert.deepEqual(receipt.command, { executable: process.execPath, args, cwd: root });
    assert.deepEqual(receipt.result, {
      pid: result.pid,
      status: 1,
      signal: null,
      error: null,
      stdoutCaptured: true,
      stderrCaptured: true,
    });
    const files = JSON.parse(fs.readFileSync(path.join(evidence, "files.json"), "utf8"));
    assert.deepEqual(
      files.map((file: { path: string }) => file.path),
      ["stdout.bin", "stderr.bin", "command.json", "fixture/original.bin"],
    );
    for (const file of files) {
      const bytes = fs.readFileSync(path.join(evidence, file.path));
      assert.equal(bytes.length, file.sizeBytes);
      assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
    }
    assert.equal(fs.existsSync(path.join(evidence, "fixture/missing-report.json")), false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("fixture custody retains spawn errors without assuming a report exists", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custody-error-"));
  try {
    const executable = path.join(root, "missing-executable");
    const result = spawnSync(executable, [], { encoding: "buffer" });
    const evidence = retainFixtureCommand(
      path.join(root, "retained"),
      { executable, args: [], cwd: root, timeoutMs: 20_000 },
      result,
      {},
    );
    const receipt = JSON.parse(fs.readFileSync(path.join(evidence, "command.json"), "utf8"));
    assert.equal(receipt.result.status, null);
    assert.equal(receipt.result.error.code, "ENOENT");
    assert.equal(receipt.result.error.message, result.error!.message);
    assert.equal(receipt.result.error.stack, result.error!.stack);
    assert.equal(receipt.result.stdoutCaptured, false);
    assert.equal(receipt.result.stderrCaptured, false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
