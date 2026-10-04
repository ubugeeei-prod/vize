import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const capture = process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_CAPTURE;
assert(capture && process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_REQUIRE_CAPTURE === "1");
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const attempts: any[] = [];
for (const mode of ["development", "production"]) {
  const prefix = path.join(path.dirname(capture), `native-original-for-constant-sfc-dom-${mode}`);
  const args = [
    "node",
    "--test",
    "tests/tooling/native-original-for-constant-sfc-dom-reference.test.ts",
  ];
  const child = spawnSync("vp", args, {
    cwd: process.cwd(),
    timeout: 90_000,
    maxBuffer: 32 * 1024 * 1024,
    env: {
      ...process.env,
      NODE_ENV: mode,
      VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_RUNTIME_CAPTURE: `${prefix}-runtime.json`,
    },
  });
  const stdout = child.stdout ?? Buffer.alloc(0),
    stderr = child.stderr ?? Buffer.alloc(0);
  const receipt = {
    mode,
    command: ["vp", ...args],
    exitStatus: child.status,
    signal: child.signal,
    processError: child.error?.message ?? null,
    stdoutBase64: stdout.toString("base64"),
    stderrBase64: stderr.toString("base64"),
    stdoutSha256: hash(stdout),
    stderrSha256: hash(stderr),
    sourceCaptureHash: hash(fs.readFileSync(capture)),
  };
  fs.writeFileSync(`${prefix}.stdout`, stdout);
  fs.writeFileSync(`${prefix}.stderr`, stderr);
  fs.writeFileSync(`${prefix}.process.json`, JSON.stringify(receipt, null, 2) + "\n");
  attempts.push(receipt);
}
// Preserve BOTH raw processes before judging either exit, without retry/fallback.
for (const attempt of attempts) {
  assert.equal(attempt.exitStatus, 0, `${attempt.mode}: real whole-module runtime process failed`);
  assert.equal(attempt.signal, null);
  assert.equal(attempt.processError, null);
  const prefix = path.join(
    path.dirname(capture),
    `native-original-for-constant-sfc-dom-${attempt.mode}`,
  );
  assert(fs.statSync(`${prefix}-runtime.json`).size > 0);
}
