import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { arg, packet, type ProcessPacket } from "./utils.ts";

interface ConfigRow {
  id: string;
  directory: string;
  actual: ProcessPacket;
  expected?: ProcessPacket;
  expectedReport?: Record<string, unknown>;
  equal: boolean;
  beforeEqual?: boolean;
  error?: string;
}

const workspace = arg(0);
const out = arg(1);
const cli = path.join(workspace, "node_modules/.bin/vize");
const env = { ...process.env, NODE_OPTIONS: "--stack-trace-limit=0" };
const CONFIG =
  "export default async () => { await Promise.resolve(); throw new TypeError('owned-check-config-recovery'); };\n";
const CLEAN =
  '<script setup lang="ts">\nconst n: number = 1;\n</script>\n<template><div>{{ n }}</div></template>\n';
const BROKEN =
  "<script setup lang=\"ts\">\nconst n: number = 'wrong';\n</script>\n<template><div>{{ n }}</div></template>\n";
const tick = String.fromCharCode(96);
const HINT =
  "Hint: use " +
  tick +
  "vize check --no-config" +
  tick +
  " to type-check without loading project configuration.\n";
const probe = spawnSync(
  process.execPath,
  [
    "--input-type=module",
    "-e",
    "process.stdout.write(process.version); await Promise.resolve(); throw new TypeError('owned-refusal-probe');",
  ],
  { env, encoding: "utf8" },
);
assert.equal(probe.status, 1);
assert.equal(probe.stderr.split("[TypeError: owned-refusal-probe]").length, 2);
const body = probe.stderr.replace(
  "[TypeError: owned-refusal-probe]",
  "[TypeError: owned-check-config-recovery]",
);
const rows: ConfigRow[] = [];
let root = "";
const samePacket = (a: ProcessPacket, b: ProcessPacket): boolean =>
  JSON.stringify(a) === JSON.stringify(b);
for (const filename of ["vite.config.mjs", "vite.config.ts"]) {
  root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "vize-public-config-recovery-")));
  fs.writeFileSync(path.join(root, filename), CONFIG);
  fs.writeFileSync(path.join(root, "A.vue"), CLEAN);
  for (const explicit of [false, true]) {
    const actual = packet(
      spawnSync(cli, ["check", "--format", "json", ...(explicit ? ["--config", filename] : [])], {
        cwd: root,
        env,
        encoding: "utf8",
        timeout: 30000,
      }),
    );
    const esc = String.fromCharCode(27);
    const before: ProcessPacket = {
      status: 2,
      signal: null,
      stdout: "",
      stderr:
        esc +
        "[31mError:" +
        esc +
        "[0m failed to parse " +
        (explicit ? filename : path.join(root, filename)) +
        ": node failed to load config: " +
        body,
    };
    const expected: ProcessPacket = { ...before, stderr: before.stderr + HINT };
    rows.push({
      id: filename + (explicit ? "-explicit" : "-discovered"),
      directory: root,
      actual,
      expected,
      equal: samePacket(actual, expected),
      beforeEqual: samePacket(actual, before),
    });
    assert.equal(fs.readFileSync(path.join(root, filename), "utf8"), CONFIG);
    assert.equal(fs.readFileSync(path.join(root, "A.vue"), "utf8"), CLEAN);
  }
}
fs.symlinkSync(path.join(workspace, "node_modules"), path.join(root, "node_modules"), "dir");
const compilerOptions = {
  strict: true,
  target: "ES2022",
  module: "ESNext",
  moduleResolution: "bundler",
  noEmit: true,
};
fs.writeFileSync(
  path.join(root, "tsconfig.json"),
  JSON.stringify({ compilerOptions, include: ["A.vue"] }),
);
const cases: Array<[string, string, number, string[]]> = [
  ["clean", CLEAN, 0, []],
  ["broken", BROKEN, 1, ["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."]],
];
for (const [id, source, status, diagnostics] of cases) {
  fs.writeFileSync(path.join(root, "A.vue"), source);
  const actual = packet(
    spawnSync(cli, ["check", "--no-config", "--format", "json"], {
      cwd: root,
      env,
      encoding: "utf8",
      timeout: 30000,
    }),
  );
  const expectedReport: Record<string, unknown> = {
    files: [{ file: "A.vue", diagnostics }],
    programs: [{ root: ".", tsconfig: "tsconfig.json", compilerOptions, files: ["A.vue"] }],
    errorCount: status,
    warningCount: 0,
    fileCount: 1,
  };
  let equal = true;
  let error: string | undefined;
  try {
    assert.equal(actual.status, status);
    assert.equal(actual.signal, null);
    assert.equal(
      actual.stderr,
      "Building Corsa virtual project for 1 files under " +
        root +
        "...\nRunning Corsa diagnostics for 1 files...\n",
    );
    assert.deepEqual(JSON.parse(actual.stdout), expectedReport);
  } catch (caught) {
    equal = false;
    error = caught instanceof Error ? caught.message : String(caught);
  }
  rows.push({
    id: "no-config-" + id,
    directory: root,
    actual,
    expectedReport,
    equal,
    ...(error ? { error } : {}),
  });
  assert.equal(fs.readFileSync(path.join(root, "vite.config.ts"), "utf8"), CONFIG);
  assert.equal(fs.readFileSync(path.join(root, "A.vue"), "utf8"), source);
}
fs.writeFileSync(
  out,
  JSON.stringify({ workspace, node: process.version, oracle: packet(probe), rows }, null, 2) + "\n",
);
console.log(
  JSON.stringify(
    rows.map(({ id, equal, beforeEqual, error }) => ({ id, equal, beforeEqual, error })),
  ),
);
assert.ok(rows.every((row) => row.equal));
