import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { captureProjectCommand } from "./project-command-capture.mjs";

test("same actual command/argv/cwd/env/stdin retains complete binary and UTF8 results", async (t) => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-command-capture-"));
  t.after(() => fs.rmSync(cwd, { recursive: true, force: true }));
  const args = [
    "-e",
    `const fs=require('node:fs');process.stdout.write(fs.readFileSync(0));process.stdout.write(JSON.stringify([process.argv.slice(1),process.cwd(),process.env.VIZE_CAPTURE]));process.stderr.write(Buffer.from([0,255,228,184,173]));process.exitCode=7;`,
    "whole",
    "日本語",
  ];
  for (const encoding of [undefined, "utf8"]) {
    const options = {
      cwd,
      env: { ...process.env, VIZE_CAPTURE: "exact" },
      input: Buffer.from([0, 255, 228, 184, 173]),
      timeout: 3000,
      maxBuffer: 1024,
      ...(encoding ? { encoding } : {}),
    };
    const original = spawnSync(process.execPath, args, options);
    const actual = await captureProjectCommand(process.execPath, args, options);
    for (const key of ["status", "signal", "stdout", "stderr", "output", "error"])
      assert.deepEqual(actual[key], original[key], key);
  }
});

test("spawn failure and combined output overflow remain fatal complete outcomes", async () => {
  const missing = await captureProjectCommand("vize-no-such-project-command", []);
  assert.equal(missing.error.code, "ENOENT");
  assert.notEqual(missing.status, 0);
  const overflow = await captureProjectCommand(
    process.execPath,
    [
      "-e",
      "process.stdout.write('abcdefgh');process.stderr.write('ijklmnop');setInterval(()=>{},1000)",
    ],
    { maxBuffer: 12, timeout: 3000 },
  );
  assert.equal(overflow.error.code, "ENOBUFS");
  assert.ok(overflow.stdout.length + overflow.stderr.length > 12);
  assert.notEqual(overflow.status, 0);
});

test("timeout escalates for ignored SIGTERM and drains descendants after leader closure", async () => {
  const script = `const {spawn}=require('node:child_process');const child=spawn(process.execPath,['-e',"process.on('SIGTERM',()=>{});setInterval(()=>{},1000)"],{stdio:'ignore'});process.stdout.write(JSON.stringify({pid:process.pid,child:child.pid})+'\\n');process.on('SIGTERM',()=>{});setInterval(()=>{},1000);`;
  const result = await captureProjectCommand(process.execPath, ["-e", script], {
    timeout: 500,
    maxBuffer: 1024,
  });
  assert.equal(result.error.code, "ETIMEDOUT");
  assert.equal(result.signal, "SIGKILL");
  const ids = JSON.parse(result.stdout.toString());
  for (const pid of [ids.pid, ids.child]) {
    const state = spawnSync("ps", ["-p", String(pid), "-o", "stat="], { encoding: "utf8" });
    assert.ok(state.status === 1 || state.stdout.trim().startsWith("Z"));
  }
  const normal = await captureProjectCommand(
    process.execPath,
    [
      "-e",
      script.replace(
        "process.on('SIGTERM',()=>{});setInterval(()=>{},1000);",
        "process.exitCode=0;setTimeout(()=>process.exit(0),10);",
      ),
    ],
    { timeout: 3000, maxBuffer: 1024 },
  );
  assert.equal(normal.status, 0);
  assert.equal(normal.error, undefined);
  const descendant = JSON.parse(normal.stdout.toString()).child;
  const state = spawnSync("ps", ["-p", String(descendant), "-o", "stat="], { encoding: "utf8" });
  assert.ok(state.status === 1 || state.stdout.trim().startsWith("Z"));
});
