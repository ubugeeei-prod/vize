import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { normalizeOxlintObservation } from "../../../tools/support/release/smoke_release_install/runtime.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const smokeCommand = path.join(root, "tools/commands/release/npm/smoke-release-install.ts");

test("the Bun lane executes the shared source planner when its graph changes", () => {
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/release-smoke-contracts.yml"),
    "utf8",
  );
  for (const source of [
    "tools/support/compat/github/legacy-fix-fixture-report.ts",
    "tools/support/compat/github/native-beta-dto-fixtures.ts",
    "tools/support/compat/github/report-legacy-fix-fixtures.ts",
    "tests/tooling/legacy-fix-fixture-report.test.ts",
  ]) {
    assert.ok(workflow.includes(`- "${source}"`), `${source} must select the Bun lane`);
  }
  assert.match(workflow, /name: Shared source planner on Bun 1\.3\.14/);
  assert.match(workflow, /bun test tests\/tooling\/legacy-fix-fixture-report\.test\.ts/);
  assert.match(workflow, /name: Shared source planner on Node 24\.14\.0/);
  assert.match(workflow, /node --test tests\/tooling\/legacy-fix-fixture-report\.test\.ts/);
  assert.match(workflow, /bun-planner-\$phase\.txt/);
  assert.match(workflow, /node-planner-\$phase\.txt/);
});

test("Bun install and Node/Bun parity execute through the configured runtime", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize bun runtime smoke test-"));
  try {
    const log = path.join(tempDir, "runtime.jsonl");
    const resolvedNpm = resolvePathCommand("npm");
    const realNpm =
      process.platform === "win32"
        ? writeWindowsCommandProxy(tempDir, "real npm.cmd", resolvedNpm)
        : resolvedNpm;
    const npmStub = writeRuntimeStub(tempDir, "npm");
    const bunStub = writeRuntimeStub(tempDir, "bun");
    const nativeDir = writeNativePackage(tempDir);
    const packOnlyDir = writePackOnlyPackage(tempDir);
    const environment = {
      BUN_BIN: bunStub,
      NPM_BIN: npmStub,
      NODE_OPTIONS:
        process.platform === "win32"
          ? [process.env.NODE_OPTIONS, `--require="${path.join(tempDir, "bun-runtime-hook.cjs")}"`]
              .filter(Boolean)
              .join(" ")
          : process.env.NODE_OPTIONS,
      VIZE_RUNTIME_STUB_LOG: log,
      VIZE_RUNTIME_STUB_NPM: realNpm,
    };

    const matching = runSmoke(
      ["--runtime-checks", "--bun-runtime-checks", nativeDir, packOnlyDir],
      environment,
    );
    assert.equal(matching.status, 0, `${matching.stderr}\n${matching.stdout}`.trim());
    assert.match(matching.stdout, /Bun and Node diagnostics\/generated output are identical/);
    const invocations = fs
      .readFileSync(log, "utf8")
      .trim()
      .split("\n")
      .map(
        (line) =>
          JSON.parse(line) as {
            args: string[];
            cwd: string;
            kind: string;
            overrides?: Record<string, string>;
          },
      );
    assert.ok(
      invocations.some(
        ({ args, kind }) =>
          kind === "bun" && args[0] === "install" && args[1] === "--ignore-scripts",
      ),
      "Bun must install the packed graph without lifecycle scripts",
    );
    const bunInstall = invocations.find(
      ({ args, kind }) => kind === "bun" && args[0] === "install",
    );
    assert.deepEqual(Object.keys(bunInstall?.overrides ?? {}), [
      "@vizejs/native",
      "@vizejs/native-pack-only",
    ]);
    const nativeOverride = bunInstall!.overrides!["@vizejs/native"];
    assert.ok(nativeOverride.startsWith("file:"));
    assert.equal(
      path.resolve(bunInstall!.cwd, nativeOverride.slice("file:".length)),
      path.resolve(bunInstall!.cwd, "../packs/vizejs-native-1.0.0.tgz"),
    );
    assert.ok(
      invocations.some(({ args, kind }) => kind === "bun" && args[0] === "-e"),
      "the packed native product must execute through the configured Bun runtime",
    );

    fs.writeFileSync(log, "");
    const differing = runSmoke(
      ["--runtime-checks", "--bun-runtime-checks", nativeDir, packOnlyDir],
      {
        ...environment,
        VIZE_BUN_STUB_DIFFER: "1",
      },
    );
    assert.equal(differing.status, 1);
    assert.match(differing.stderr, /Bun runtime behavior differs from Node/);
    assert.match(differing.stderr, /\\"code\\":\\"node\\"/);
    assert.match(differing.stderr, /\\"code\\":\\"bun\\"/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("Oxlint parity excludes only per-invocation timing", () => {
  const report = {
    diagnostics: [
      {
        code: "vize(vue/no-v-html)",
        filename: "/packed/src/App.vue",
        labels: [{ span: { column: 1, length: 6, line: 1, offset: 0 } }],
        message: "unsafe html",
        severity: "warning",
      },
    ],
    number_of_files: 2,
    number_of_rules: 89,
    start_time: 12.5,
    threads_count: 4,
  };
  const first = normalizeOxlintObservation(JSON.stringify(report), "/packed");
  const second = normalizeOxlintObservation(
    JSON.stringify({ ...report, start_time: 0.000_001 }),
    "/packed",
  );

  assert.equal(first, second);
  assert.deepEqual(JSON.parse(first), {
    diagnostics: [
      {
        code: "vize(vue/no-v-html)",
        filename: "/VIZE_PROJECT/src/App.vue",
        labels: [{ span: { column: 1, length: 6, line: 1, offset: 0 } }],
        message: "unsafe html",
        severity: "warning",
      },
    ],
    number_of_files: 2,
    number_of_rules: 89,
    threads_count: 4,
  });
});

function writeNativePackage(tempDir: string): string {
  const nativeDir = path.join(tempDir, "native");
  fs.mkdirSync(nativeDir);
  fs.writeFileSync(
    path.join(nativeDir, "package.json"),
    JSON.stringify({
      exports: { ".": { import: "./index.js", require: "./index.js", types: "./index.d.ts" } },
      files: ["index.js", "index.d.ts"],
      name: "@vizejs/native",
      version: "1.0.0",
    }),
  );
  fs.writeFileSync(
    path.join(nativeDir, "index.d.ts"),
    "export declare function compileSfc(): unknown;\n",
  );
  fs.writeFileSync(
    path.join(nativeDir, "index.js"),
    `const runtime = process.env.VIZE_BUN_STUB_RUNTIME === "1" ? "bun" : "node";
module.exports.compileSfc = () => ({
  code: process.env.VIZE_BUN_STUB_DIFFER === "1" ? runtime : "identical",
  errors: [],
  warnings: [],
});
`,
  );
  return nativeDir;
}

function writePackOnlyPackage(tempDir: string): string {
  const packageDir = path.join(tempDir, "native-pack-only");
  fs.mkdirSync(packageDir);
  fs.writeFileSync(
    path.join(packageDir, "package.json"),
    JSON.stringify({
      cpu: ["vize-never"],
      exports: { ".": { import: "./index.js", types: "./index.d.ts" } },
      files: ["index.js", "index.d.ts"],
      name: "@vizejs/native-pack-only",
      version: "1.0.0",
    }),
  );
  fs.writeFileSync(path.join(packageDir, "index.d.ts"), "export declare const packed: true;\n");
  fs.writeFileSync(path.join(packageDir, "index.js"), "export const packed = true;\n");
  return packageDir;
}

function runSmoke(args: string[], env: NodeJS.ProcessEnv) {
  return spawnSync(process.execPath, [smokeCommand, ...args], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...env },
  });
}

function resolvePathCommand(name: string): string {
  const extensions =
    process.platform === "win32" ? (process.env.PATHEXT ?? ".CMD;.EXE").split(";") : [""];
  for (const directory of (process.env.PATH ?? "").split(path.delimiter))
    for (const extension of extensions) {
      const candidate = path.join(directory, `${name}${extension.toLowerCase()}`);
      try {
        if (fs.statSync(candidate).isFile()) return fs.realpathSync.native(candidate);
      } catch {
        // Try the next PATH entry.
      }
    }
  throw new Error(`${name} is not available on PATH`);
}

function writeRuntimeStub(tempDir: string, kind: "bun" | "npm"): string {
  if (process.platform === "win32" && kind === "bun") return writeWindowsBunRuntimeStub(tempDir);
  const script = path.join(tempDir, `${kind}-runtime-stub.cjs`);
  fs.writeFileSync(
    script,
    `#!${process.execPath}
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const kind = ${JSON.stringify(kind)};
const args = process.argv.slice(2);
const originalArgs = [...args];
const installing = args[0] === "install";
let overrides;
if (installing) {
  const manifestPath = path.join(process.cwd(), "package.json");
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  overrides = manifest.overrides;
  manifest.dependencies = Object.fromEntries(
    Object.entries(manifest.dependencies || {}).filter(([, value]) => String(value).startsWith("file:")),
  );
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\\n");
  args.splice(0, args.length, "install", "--ignore-scripts", "--package-lock=false", "--no-audit", "--fund=false", "--legacy-peer-deps", "--include=optional");
}
fs.appendFileSync(process.env.VIZE_RUNTIME_STUB_LOG, JSON.stringify({ kind, args: originalArgs, cwd: process.cwd(), overrides }) + "\\n");
const command = kind === "npm" || installing ? process.env.VIZE_RUNTIME_STUB_NPM : process.execPath;
const env = kind === "bun" && !installing
  ? { ...process.env, VIZE_BUN_STUB_RUNTIME: "1" }
  : process.env;
const shell = process.platform === "win32" && /\\.(cmd|bat)$/i.test(command);
const executable = shell ? '"' + command + '"' : command;
const child = spawnSync(executable, args, {
  cwd: process.cwd(),
  env,
  shell,
  stdio: "inherit",
});
if (child.error) throw child.error;
process.exit(child.status ?? 1);
`,
  );
  if (process.platform !== "win32") {
    fs.chmodSync(script, 0o755);
    return script;
  }
  const wrapper = path.join(tempDir, `${kind}-runtime-stub.cmd`);
  fs.writeFileSync(wrapper, `@echo off\r\n"${process.execPath}" "${script}" %*\r\n`);
  return wrapper;
}

function writeWindowsBunRuntimeStub(tempDir: string): string {
  const executable = path.join(tempDir, "bun-runtime-stub.exe");
  fs.copyFileSync(process.execPath, executable);
  fs.writeFileSync(
    path.join(tempDir, "bun-runtime-hook.cjs"),
    `const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const stub = ${JSON.stringify(executable)}.toLowerCase();
if (process.execPath.toLowerCase() === stub) {
  const installing = path.basename(process.argv[1] || "").toLowerCase() === "install";
  const args = installing ? ["install", ...process.argv.slice(2)] : process.execArgv;
  let overrides;
  if (installing) {
    const manifestPath = path.join(process.cwd(), "package.json");
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    overrides = manifest.overrides;
    manifest.dependencies = Object.fromEntries(
      Object.entries(manifest.dependencies || {}).filter(([, value]) => String(value).startsWith("file:")),
    );
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\\n");
    fs.appendFileSync(process.env.VIZE_RUNTIME_STUB_LOG, JSON.stringify({ kind: "bun", args, cwd: process.cwd(), overrides }) + "\\n");
    const child = spawnSync(
      '"' + process.env.VIZE_RUNTIME_STUB_NPM + '"',
      ["install", "--ignore-scripts", "--package-lock=false", "--no-audit", "--fund=false", "--legacy-peer-deps", "--include=optional"],
      { cwd: process.cwd(), env: process.env, shell: true, stdio: "inherit" },
    );
    if (child.error) throw child.error;
    process.exit(child.status ?? 1);
  }
  fs.appendFileSync(process.env.VIZE_RUNTIME_STUB_LOG, JSON.stringify({ kind: "bun", args, cwd: process.cwd(), overrides }) + "\\n");
  process.env.VIZE_BUN_STUB_RUNTIME = "1";
}
`,
  );
  return executable;
}

function writeWindowsCommandProxy(tempDir: string, name: string, command: string): string {
  const proxy = path.join(tempDir, name);
  fs.writeFileSync(proxy, `@call "${command}" %*\r\n`);
  return proxy;
}
