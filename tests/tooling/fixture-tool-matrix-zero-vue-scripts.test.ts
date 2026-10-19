import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  collectOwnedScriptFiles,
  prepareScriptSourceScope,
} from "../../tools/support/compat/fixtures/tool-matrix-inputs.ts";
import { runTool } from "../../tools/support/compat/fixtures/tool-matrix-run.mjs";
import { toolArgs } from "../../tools/support/compat/fixtures/tool-matrix-command.mjs";
import { validateTypecheckerOutput } from "../../tools/support/compat/fixtures/tool-matrix-typechecker.mjs";
import { resolveVizeBin } from "./support/check-fixtures/cycle-targets.ts";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const project = { id: "scripts", expectedVueFileCount: 0, vueGlobs: ["**/*.vue"] };
const configName = ".vize-fixture-typecheck-scripts.tsconfig.json";
const typeError = "Type 'string' is not assignable to type 'number'.";

function fixture() {
  const directory = fs.mkdtempSync(path.join(root, "tests/_fixtures/matrix-owned-scripts-"));
  fs.mkdirSync(path.join(directory, "src"));
  fs.writeFileSync(
    path.join(directory, "src/entry.js"),
    '/** @type {number} */\nexport const value = "wrong";\n',
  );
  fs.writeFileSync(
    path.join(directory, "src/second.ts"),
    'export const second: number = "wrong";\n',
  );
  fs.writeFileSync(path.join(directory, "src/types.d.ts"), "declare const preserved: string;\n");
  fs.writeFileSync(path.join(directory, "src/Ignored.JS"), "broken uppercase extension\n");
  return directory;
}

function report(scope: any) {
  return {
    files: scope.roots.map((file: string) => ({
      file,
      diagnostics: [`error:2:14 [TS2322] ${typeError}`],
    })),
    programs: [
      {
        root: ".",
        tsconfig: scope.tsconfig,
        compilerOptions: scope.config.compilerOptions,
        files: [...scope.roots],
      },
    ],
    errorCount: scope.roots.length,
    warningCount: 0,
    fileCount: scope.roots.length,
  };
}

test("generated zero-Vue ownership records physical scripts and preserves complete diagnostics", () => {
  const directory = fixture();
  try {
    const scope = prepareScriptSourceScope(directory, configName);
    assert.deepEqual(scope.config, {
      compilerOptions: { allowJs: true, checkJs: true, jsx: "preserve" },
      files: ["src/entry.js", "src/second.ts", "src/types.d.ts"],
      include: [],
    });
    assert.deepEqual(scope.roots, ["src/entry.js", "src/second.ts"]);
    for (const record of scope.files) {
      const bytes = fs.readFileSync(path.join(directory, record.file));
      assert.equal(record.bytes, bytes.length);
      assert.equal(record.sha256, createHash("sha256").update(bytes).digest("hex"));
    }
    const output = report(scope);
    const original = structuredClone(output);
    const coverage = validateTypecheckerOutput(project, output, 1, [], scope.config.files, scope);
    assert.deepEqual(output, original);
    assert.equal(coverage.requested.fileCount, 0);
    assert.deepEqual(coverage.transitiveAuthored.files, scope.roots);
    assert.equal(
      coverage.checked.sha256,
      createHash("sha256").update("src/entry.js\0src/second.ts\0").digest("hex"),
    );
    assert.throws(
      () => validateTypecheckerOutput(project, output, 1, [], scope.config.files),
      /expected zero/,
    );
    assert.deepEqual(
      toolArgs(project, "typechecker", "unused", { typecheckTsconfigPath: configName }),
      ["check", "--format", "json", "--no-config", "--tsconfig", configName],
    );
    assert.deepEqual(
      toolArgs({ ...project, expectedVueFileCount: 1 }, "typechecker", "unused", {
        typecheckTsconfigPath: configName,
      }).slice(0, 2),
      ["check", "**/*.vue"],
    );
    assert.deepEqual(
      toolArgs({ ...project, tsconfig: "jsconfig.json" }, "typechecker", "unused", {
        typecheckTsconfigPath: "jsconfig.json",
      }).slice(0, 2),
      ["check", "**/*.vue"],
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("generated script ownership rejects missing roots, foreign programs and altered custody", () => {
  const directory = fixture();
  try {
    const scope = prepareScriptSourceScope(directory, configName);
    for (const mutate of [
      (r: any) => {
        r.files.pop();
        r.fileCount -= 1;
        r.errorCount -= 1;
      },
      (r: any) => {
        r.programs[0].root = "../foreign";
      },
      (r: any) => {
        r.programs[0].tsconfig = "foreign.json";
      },
      (r: any) => {
        r.programs[0].compilerOptions.checkJs = false;
      },
      (r: any) => {
        r.programs[0].files.pop();
      },
      (r: any) => {
        r.programs[0].files.push("unknown.ts");
      },
      (r: any) => {
        r.files[0].file = "../escaping.js";
      },
      (r: any) => {
        r.errorCount = 0;
      },
    ]) {
      const r = structuredClone(report(scope));
      mutate(r);
      assert.throws(() => validateTypecheckerOutput(project, r, 1, [], scope.config.files, scope));
    }
    for (const mutate of [
      (s: any) => {
        s.roots.pop();
      },
      (s: any) => {
        s.config.include.push("../foreign/**");
      },
      (s: any) => {
        s.configSha256 = "0".repeat(64);
      },
      (s: any) => {
        s.files[0].file = "src/entry.JS";
      },
      (s: any) => {
        s.files[0].bytes = -1;
      },
      (s: any) => {
        s.files[0].sha256 = "bad";
      },
      (s: any) => {
        s.extra = "unknown";
      },
    ]) {
      const s = structuredClone(scope);
      mutate(s);
      assert.throws(() =>
        validateTypecheckerOutput(project, report(scope), 1, [], scope.config.files, s),
      );
    }
    assert.throws(() =>
      validateTypecheckerOutput(
        { ...project, expectedVueFileCount: 1 },
        report(scope),
        1,
        [],
        scope.config.files,
        scope,
      ),
    );
    assert.throws(() =>
      validateTypecheckerOutput(
        { ...project, tsconfig: "own.json" },
        report(scope),
        1,
        [],
        scope.config.files,
        scope,
      ),
    );
    assert.throws(() =>
      validateTypecheckerOutput(
        project,
        { files: [], programs: [], errorCount: 0, warningCount: 0, fileCount: 0 },
        0,
        [],
        scope.config.files,
        scope,
      ),
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("physical source scan excludes metadata and rejects escaping ownership", () => {
  const directory = fixture();
  const foreign = fs.mkdtempSync(path.join(os.tmpdir(), "vize-foreign-script-"));
  try {
    for (const name of [".git", ".yarn", "node_modules"]) {
      fs.mkdirSync(path.join(directory, name));
      fs.writeFileSync(path.join(directory, name, "excluded.js"), "unknown\n");
    }
    assert.deepEqual(collectOwnedScriptFiles(directory), [
      "src/entry.js",
      "src/second.ts",
      "src/types.d.ts",
    ]);
    fs.writeFileSync(path.join(foreign, "foreign.js"), "export {};\n");
    fs.symlinkSync(path.join(foreign, "foreign.js"), path.join(directory, "escape.js"));
    assert.throws(() => prepareScriptSourceScope(directory, configName), /escapes root/);
    fs.unlinkSync(path.join(directory, "escape.js"));
    fs.symlinkSync(directory, path.join(directory, "cycle"), "dir");
    assert.throws(() => prepareScriptSourceScope(directory, configName), /cycles/);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
    fs.rmSync(foreign, { recursive: true, force: true });
  }
});

test("actual native zero-Vue matrix checks both broken physical roots under its generated owner", () => {
  const directory = fixture();
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-script-native-"));
  try {
    const binary = resolveVizeBin(root);
    assert.ok(fs.existsSync(binary), `native source CLI is required: ${binary}`);
    assert.equal(path.resolve(binary), path.join(root, "target/ci/vize"));
    const producer = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
    validateBuildReceipt(producer, expectedBuildIdentity(root), producer.recipe);
    const beforeConfig = '{"compilerOptions":{}}\n';
    fs.writeFileSync(path.join(directory, configName), beforeConfig);
    const beforeArgs = [
      "check",
      "**/*.vue",
      "--format",
      "json",
      "--no-config",
      "--tsconfig",
      configName,
    ];
    const before = spawnSync(binary, beforeArgs, {
      cwd: directory,
      encoding: "utf8",
      timeout: 60_000,
    });

    const run = runTool(
      { ...project, fixturePath: path.relative(root, directory) },
      "typechecker",
      { dryRun: false, timeoutMs: 60_000 },
      { command: binary, prefix: [], label: binary },
      outputDir,
    );
    const raw = JSON.parse(
      fs.readFileSync(path.join(outputDir, "scripts-typechecker.json"), "utf8"),
    );
    const packet = path.join(root, "target/differential/fixture-tool-matrix-zero-vue-scripts.json");
    fs.mkdirSync(path.dirname(packet), { recursive: true });
    fs.writeFileSync(
      packet,
      JSON.stringify(
        {
          schema: "vize.fixtureScriptSourceAcceptance",
          version: 1,
          producer,
          before: {
            args: beforeArgs,
            config: beforeConfig,
            exitCode: before.status,
            stdout: before.stdout,
            stderr: before.stderr,
          },
          run,
          report: raw,
        },
        null,
        2,
      ) + "\n",
    );
    assert.equal(before.status, 2, before.stderr);
    assert.deepEqual(JSON.parse(before.stdout), {
      files: [],
      programs: [],
      errorCount: 0,
      warningCount: 0,
      fileCount: 0,
    });
    assert.match(before.stderr, /No supported source files were selected/);
    assert.equal(run.status, "findings", JSON.stringify(run.failure));
    assert.deepEqual(JSON.parse(raw.stdout), raw.parsed);
    assert.deepEqual(raw.typecheckerSourceScan.roots, ["src/entry.js", "src/second.ts"]);
    assert.deepEqual(raw.typecheckerCoverage.checked.files, [
      "src/entry.js",
      "src/second.ts",
      "src/types.d.ts",
    ]);
    assert.deepEqual(raw.parsed.files, [
      {
        file: "src/entry.js",
        diagnostics: [`error:2:14 [TS2322] ${typeError}`],
      },
      {
        file: "src/second.ts",
        diagnostics: [`error:1:14 [TS2322] ${typeError}`],
      },
      { file: "src/types.d.ts", diagnostics: [] },
    ]);
    assert.equal(raw.parsed.warningCount, 0);
    assert.equal(raw.parsed.fileCount, 3);
    assert.equal(raw.parsed.errorCount, 2);
    assert.equal(fs.existsSync(path.join(directory, configName)), false);
    const original = structuredClone(raw.parsed);
    validateTypecheckerOutput(
      project,
      raw.parsed,
      raw.exitCode,
      [],
      raw.typecheckerSourceScan.config.files,
      raw.typecheckerSourceScan,
    );
    assert.deepEqual(raw.parsed, original);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("canonical Rust observer executes physical script ownership laws", () => {
  const result = spawnSync(
    "rust-script",
    ["--test", path.join(root, "tools/commands/fixtures/tool-matrix-report.rs")],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr + result.stdout);
});
