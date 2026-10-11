import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { captureProjectCommand } from "./project-command-capture.mjs";

void test("same actual command/argv/cwd/env/stdin retains complete binary and UTF8 results", async (t) => {
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

void test("spawn failure and combined output overflow remain fatal complete outcomes", async () => {
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

void test("timeout escalates for ignored SIGTERM and drains descendants after leader closure", async () => {
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

for (const mode of ["cancel", "missing-object", "missing-command"])
  void test(`real reference ${mode} keeps fatal cancellation separate from the pinned fetch fallback`, async (t) => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reference-command-control-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    fs.mkdirSync(path.join(root, "bin"));
    fs.mkdirSync(path.join(root, "output"));
    if (mode !== "missing-command")
      fs.writeFileSync(
        path.join(root, "bin/git"),
        `#!${process.execPath}
const fs=require('node:fs'),root=process.env.VIZE_REFERENCE_CONTROL,args=process.argv.slice(2);
fs.appendFileSync(root+'/commands.jsonl',JSON.stringify({pid:process.pid,args})+'\\n');
if(args[0]==='cat-file') {
 process.stdout.write('whole exists stdout');process.stderr.write('whole exists stderr');
 if(process.env.VIZE_REFERENCE_MODE==='cancel') {fs.writeFileSync(root+'/ready',String(process.pid));setInterval(()=>{},1000);}
 else process.exitCode=128;
} else if(args[0]==='fetch') process.exitCode=0;
else {process.stdout.write('refuse invalid tree authority');process.exitCode=0;}
`,
        { mode: 0o755 },
      );
    const moduleUrl = (name) => JSON.stringify(new URL(name, import.meta.url).href);
    const driver = path.join(root, "driver.mjs");
    fs.writeFileSync(
      driver,
      `import fs from 'node:fs';
import {prepareReferenceSource} from ${moduleUrl("./n8n-reference-source.mjs")};
import {runQualificationBranches} from ${moduleUrl("./project-qualification-branches.mjs")};
const root=${JSON.stringify(root)},mode=${JSON.stringify(mode)};
const timer=mode==='cancel'?setInterval(()=>{if(fs.existsSync(root+'/ready')){clearInterval(timer);process.kill(process.pid,'SIGTERM');}},10):null;
try {await runQualificationBranches([
 {id:'mixedDirectory8507',run:()=>new Promise(resolve=>setTimeout(()=>{fs.writeFileSync(root+'/remaining.done','complete');resolve();},150))},
 {id:'n8n',run:()=>prepareReferenceSource(root+'/output')},
],root+'/branch-outcomes.json');process.exitCode=2;}
catch {process.exitCode=1;}finally{clearInterval(timer);}
`,
    );
    const child = spawn(process.execPath, [driver], {
      env: {
        ...process.env,
        PATH: path.join(root, "bin"),
        GITHUB_ACTIONS: "true",
        VIZE_REFERENCE_CONTROL: root,
        VIZE_REFERENCE_MODE: mode,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "",
      stderr = "";
    child.stdout.on("data", (bytes) => (stdout += bytes));
    child.stderr.on("data", (bytes) => (stderr += bytes));
    const timeout = setTimeout(() => child.kill("SIGKILL"), 5000);
    t.after(() => clearTimeout(timeout));
    const terminal = await new Promise((resolve, reject) => {
      child.once("error", reject);
      child.once("close", (status, signal) => resolve({ status, signal }));
    });
    assert.deepEqual(terminal, { status: 1, signal: null }, stdout + stderr);
    const packet = JSON.parse(fs.readFileSync(path.join(root, "output/reference-exists.json")));
    const outcomes = JSON.parse(fs.readFileSync(path.join(root, "branch-outcomes.json")));
    assert.equal(outcomes.interrupted, mode === "cancel" ? "SIGTERM" : null);
    assert.deepEqual(
      outcomes.branches.map(({ status }) => status),
      ["fulfilled", "rejected"],
    );
    assert.ok(fs.existsSync(path.join(root, "remaining.done")));
    const commands = fs.existsSync(path.join(root, "commands.jsonl"))
      ? fs
          .readFileSync(path.join(root, "commands.jsonl"), "utf8")
          .trim()
          .split("\n")
          .map(JSON.parse)
      : [];
    assert.deepEqual(
      commands.map(({ args }) => args[0]),
      mode === "missing-object"
        ? ["cat-file", "fetch", "rev-parse"]
        : mode === "cancel"
          ? ["cat-file"]
          : [],
    );
    if (mode === "missing-object") {
      assert.equal(packet.status, 128);
      assert.equal(packet.signal, null);
      assert.deepEqual(commands[1].args, [
        "fetch",
        "--depth=1",
        "origin",
        "e742ca8ee730d37e7e5e47dddb0d0d08bf693389",
      ]);
    } else assert.equal(packet.error.code, mode === "cancel" ? "EINTR" : "ENOENT");
    if (mode !== "missing-command") {
      assert.equal(Buffer.from(packet.stdout).toString(), "whole exists stdout");
      assert.equal(Buffer.from(packet.stderr).toString(), "whole exists stderr");
    }
    for (const { pid } of commands) {
      const state = spawnSync("ps", ["-p", String(pid), "-o", "stat="], { encoding: "utf8" });
      assert.ok(state.status === 1 || state.stdout.trim().startsWith("Z"));
    }
  });
