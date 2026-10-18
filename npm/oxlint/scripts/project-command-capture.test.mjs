import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import { createHash } from "node:crypto";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { captureProjectCommand } from "./project-command-capture.mjs";
import {
  originalSfcSourceCommand,
  assertOriginalSfcSourceMatrix,
} from "./mixed-directory-8507-qualification.mjs";

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

// Authored regular-file controls exercise authority/refusal only: no fake addon
// is loaded and these laws cannot confer actual host or native acceptance.
function sourceMatrixAuthority(t) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-same-host-authority-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const file = (name, bytes) => {
    const physical = path.join(root, name);
    fs.writeFileSync(physical, bytes);
    return { path: physical, bytes: Buffer.byteLength(bytes), sha256: digest(bytes) };
  };
  const native = file("source.cjs", "module.exports={authoredImage:true};\n");
  const packageDir = path.join(root, "npm/oxlint");
  fs.mkdirSync(path.join(packageDir, "scripts"), { recursive: true });
  fs.mkdirSync(path.join(root, "npm/native"));
  fs.copyFileSync(
    fileURLToPath(new URL("./project-native-staging.ts", import.meta.url)),
    path.join(packageDir, "scripts/project-native-staging.ts"),
  );
  fs.writeFileSync(
    path.join(root, "npm/native/index.js"),
    "module.exports=require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);\n",
  );
  const plugin = file("plugin.mjs", "// authored plugin control\n");
  const manifest = file("host-package.json", '{"name":"oxlint","version":"1.86.0"}\n');
  const stream = file("stdout.bin", "whole output 日本語\0\n");
  const names = [
    { name: "List.vue", contrasts: ["keyed"] },
    { name: "EmptyScript.vue", contrasts: ["keyed"] },
    { name: "TplOnly.vue", contrasts: ["keyed"] },
    { name: "MyPicker.vue", contrasts: ["first-corrected", "second-corrected", "both-corrected"] },
  ].flatMap(({ name, contrasts }) =>
    ["direct", "wrapper"].flatMap((host) =>
      ["original-default", "json", ...contrasts, "rule-disabled"].map(
        (mode) => `${name}-${host}-${mode}`,
      ),
    ),
  );
  assert.equal(names.length, 36);
  const cells = names.map((name) => ({ name, stdout: stream, stderr: stream }));
  const projects = [
    {
      files: ["List.vue", ".oxlintrc.json"],
      evidence: path.join(root, "inputs"),
      before: [
        {
          filename: ".oxlintrc.json",
          archiveFilename: "1-.oxlintrc.json",
          bytes: 3,
          sha256: digest("{}\n"),
        },
      ],
    },
  ];
  projects[0].after = structuredClone(projects[0].before);
  for (const phase of ["before", "after"]) {
    fs.mkdirSync(path.join(projects[0].evidence, phase), { recursive: true });
    fs.writeFileSync(path.join(projects[0].evidence, phase, "1-.oxlintrc.json"), "{}\n");
  }
  const packet = { path: "authored immutable originals.json", sha256: digest("original controls") };
  const baseline = { hostVersion: "1.86.0", manifests: [{}, manifest], cells, projects, packet };
  return {
    root,
    packageDir,
    plugin,
    binary: native.path,
    receipt: { source: { head: "a".repeat(40) }, frozen: { sha256: native.sha256 } },
    baseline,
    matrix: {
      ...structuredClone(baseline),
      expectation: "candidate",
      contractAssertionsPassed: true,
      failures: [],
      pluginVersion: "0.463.0",
      plugin: structuredClone(plugin),
      native,
    },
    version: "0.463.0",
  };
}

void test("source command isolates overrides and uses the one existing host/addon without install or copy", async (t) => {
  const fixture = sourceMatrixAuthority(t);
  const environment = {
    GITHUB_SHA: fixture.receipt.source.head,
    GITHUB_ACTIONS: "true",
    PATH: process.env.PATH,
    NODE_OPTIONS: "--require=/foreign.cjs",
    NODE_PATH: "/foreign",
    NAPI_RS_FORCE_WASI: "true",
    NAPI_RS_NATIVE_LIBRARY_PATH: "/foreign.node",
    VIZE_OXLINT_NATIVE_CUSTODY: "/foreign.json",
    VIZE_OXLINT_TEST_ENTRYPOINT: "/foreign-oxlint",
    VIZE_PREFER_WORKSPACE_BINDING: "0",
  };
  const input = {
    ...fixture,
    packageDir: fixture.packageDir,
    installation: fixture.root,
    output: path.join(fixture.root, "candidate-1.86"),
    environment,
  };
  fs.mkdirSync(input.output);
  const actual = originalSfcSourceCommand(input);
  assert.deepEqual(actual.args, [
    path.join(fixture.packageDir, "src/test-support/original-sfc-host-matrix.ts"),
    "--plugin-package",
    fixture.packageDir,
    "--oxlint-package",
    path.join(fixture.root, "node_modules/oxlint"),
    "--output",
    input.output,
    "--expectation",
    "candidate",
  ]);
  assert.equal(actual.command, process.execPath);
  assert.equal(actual.options.cwd, fixture.root);
  assert.equal(actual.options.timeout, 180000);
  assert.equal(actual.options.maxBuffer, 64 * 1024 * 1024);
  assert.deepEqual(
    Object.fromEntries(
      Object.entries(actual.options.env).filter(
        ([key]) => key.startsWith("NAPI_") || key.startsWith("VIZE_") || key.startsWith("NODE_"),
      ),
    ),
    {
      NODE_OPTIONS: `--import=${pathToFileURL(path.join(fixture.packageDir, "scripts/project-native-staging.ts")).href}`,
      VIZE_OXLINT_NATIVE_CUSTODY: path.join(input.output, "source-preload.json"),
      VIZE_PREFER_WORKSPACE_BINDING: "1",
    },
  );
  assert.equal(actual.options.env.GITHUB_ACTIONS, undefined);
  assert.equal(environment.NODE_OPTIONS, "--require=/foreign.cjs");
  const observed = await captureProjectCommand(
    process.execPath,
    [
      "-e",
      "process.stdout.write(JSON.stringify([process.cwd(),process.env.NAPI_RS_NATIVE_LIBRARY_PATH??null,process.env.VIZE_PREFER_WORKSPACE_BINDING,process.env.NODE_OPTIONS??null]));process.stderr.write('whole stderr\\n');",
    ],
    actual.options,
  );
  assert.equal(observed.status, 0);
  assert.deepEqual(JSON.parse(observed.stdout), [
    fixture.root,
    null,
    "1",
    actual.options.env.NODE_OPTIONS,
  ]);
  assert.equal(observed.stderr.toString(), "whole stderr\n");
  assert.throws(() =>
    originalSfcSourceCommand({
      ...input,
      environment: { ...environment, GITHUB_SHA: "b".repeat(40) },
    }),
  );
  fs.appendFileSync(fixture.binary, "changed");
  assert.throws(() => originalSfcSourceCommand(input));
});

void test("same-host source receipt binds all36 cells, provider, source image and original input packet", (t) => {
  const fixture = sourceMatrixAuthority(t);
  assertOriginalSfcSourceMatrix(fixture);
  const mutations = [
    (row) => {
      row.matrix.hostVersion = "1.78.0";
    },
    (row) => {
      row.matrix.manifests[1].path = row.plugin.path;
    },
    (row) => {
      row.matrix.manifests[1].sha256 = "0".repeat(64);
    },
    (row) => {
      row.matrix.native.path = row.plugin.path;
    },
    (row) => {
      row.matrix.native.sha256 = "0".repeat(64);
    },
    (row) => {
      row.matrix.plugin.path = row.binary;
    },
    (row) => {
      row.matrix.plugin.sha256 = "0".repeat(64);
    },
    (row) => {
      row.matrix.pluginVersion = "0.458.0";
    },
    (row) => {
      row.matrix.expectation = "public-0.458";
    },
    (row) => {
      row.matrix.contractAssertionsPassed = false;
    },
    (row) => {
      row.matrix.failures = [{ stage: "native reference" }];
    },
    (row) => {
      row.matrix.cells.pop();
    },
    (row) => {
      row.matrix.cells[1].name = row.matrix.cells[0].name;
    },
    (row) => {
      row.matrix.cells[0].stdout.bytes++;
    },
    (row) => {
      row.matrix.packet.sha256 = "0".repeat(64);
    },
    (row) => {
      row.matrix.projects[0].after[0].sha256 = "0".repeat(64);
    },
  ];
  for (const [index, mutate] of mutations.entries()) {
    const changed = structuredClone(fixture);
    mutate(changed);
    assert.throws(() => assertOriginalSfcSourceMatrix(changed), `mutation${index}`);
  }
  const copied = path.join(fixture.root, "copied.node");
  fs.copyFileSync(fixture.binary, copied);
  assert.throws(() =>
    assertOriginalSfcSourceMatrix({
      ...fixture,
      matrix: { ...fixture.matrix, native: { ...fixture.matrix.native, path: copied } },
    }),
  );
  const archived = path.join(fixture.matrix.projects[0].evidence, "after/1-.oxlintrc.json");
  fs.writeFileSync(archived, "[]\n");
  assert.throws(() => assertOriginalSfcSourceMatrix(fixture));
  fs.writeFileSync(archived, "{}\n");
  fs.appendFileSync(fixture.matrix.cells[0].stdout.path, "whole-output tail mutation");
  assert.throws(() => assertOriginalSfcSourceMatrix(fixture));
});
