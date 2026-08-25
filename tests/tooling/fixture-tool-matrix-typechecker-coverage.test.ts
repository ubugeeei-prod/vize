import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { runTool } from "../../tools/fixtures/tool-matrix-run.mjs";
import {
  summarizeTypecheckerCoverage,
  validateTypecheckerOutput,
} from "../../tools/fixtures/tool-matrix-typechecker.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const project = { id: "coverage", expectedVueFileCount: 1 };

function digest(files: string[]) {
  const hash = createHash("sha256");
  for (const file of files) hash.update(file).update("\0");
  return hash.digest("hex");
}

function output(files: string[]) {
  return {
    errorCount: 0,
    warningCount: 0,
    fileCount: files.length,
    files: files.map((file) => ({ file, diagnostics: [] })),
  };
}

test("typechecker coverage partitions requested and transitive authored sources", () => {
  const requested = ["src/App.vue"];
  const transitive = ["packages/Dep.ts", "packages/render.jsx"];
  const checked = [...transitive, ...requested];
  const coverage = validateTypecheckerOutput(project, output(checked), 0, requested, checked);

  assert.deepEqual(coverage, {
    schema: "vize.fixtureTypecheckerCoverage",
    version: 1,
    requested: { fileCount: 1, files: requested, sha256: digest(requested) },
    transitiveAuthored: { fileCount: 2, files: transitive, sha256: digest(transitive) },
    checked: { fileCount: 3, files: checked, sha256: digest(checked) },
  });
  assert.deepEqual(summarizeTypecheckerCoverage(coverage), {
    requestedFileCount: 1,
    requestedSha256: digest(requested),
    transitiveAuthoredFileCount: 2,
    transitiveAuthoredSha256: digest(transitive),
    checkedFileCount: 3,
    checkedSha256: digest(checked),
  });

  const added = ["packages/Dep.ts", "packages/Extra.mts", "packages/render.jsx", "src/App.vue"];
  const expanded = validateTypecheckerOutput(project, output(added), 0, requested, added);
  assert.deepEqual(expanded.requested, coverage.requested);
  assert.notDeepEqual(expanded.transitiveAuthored, coverage.transitiveAuthored);
  assert.notDeepEqual(expanded.checked, coverage.checked);
});

test("typechecker coverage accepts the closed authored source extension set", () => {
  const requested = ["src/App.vue"];
  const checked = [
    "src/App.vue",
    "src/ambient.d.ts",
    "src/common.cjs",
    "src/common.cts",
    "src/entry.js",
    "src/entry.jsx",
    "src/module.mjs",
    "src/module.mts",
    "src/source.ts",
    "src/view.tsx",
  ];
  const coverage = validateTypecheckerOutput(project, output(checked), 0, requested, checked);
  assert.equal(coverage.transitiveAuthored.fileCount, checked.length - requested.length);
});

test("typechecker coverage fails closed on missing, substituted, or unclassified sources", () => {
  assert.throws(
    () =>
      validateTypecheckerOutput(
        project,
        output(["src/App.vue"]),
        0,
        ["src/App.vue", "src/Card.vue"],
        ["src/App.vue", "src/Card.vue"],
      ),
    /missing requested fixture inputs: \[src\/Card\.vue\]/,
  );
  assert.throws(
    () =>
      validateTypecheckerOutput(
        project,
        output(["src/App.vue"]),
        0,
        ["src/Other.vue"],
        ["src/App.vue", "src/Other.vue"],
      ),
    /missing requested fixture inputs: \[src\/Other\.vue\]/,
  );
  assert.throws(
    () =>
      validateTypecheckerOutput(
        project,
        output(["node_modules/pkg/Injected.ts", "src/App.vue"]),
        0,
        ["src/App.vue"],
        ["src/App.vue"],
      ),
    /transitive files are not authored fixture sources: \[node_modules\/pkg\/Injected\.ts\]/,
  );
  assert.throws(
    () =>
      validateTypecheckerOutput(
        project,
        output(["src/App.vue", "src/data.json"]),
        0,
        ["src/App.vue"],
        ["src/App.vue", "src/data.json"],
      ),
    /unsupported typecheck extension: src\/data\.json/,
  );
});

test("typechecker coverage rejects malformed manifests and digest mutations", () => {
  const valid = validateTypecheckerOutput(
    project,
    output(["packages/Dep.ts", "src/App.vue"]),
    0,
    ["src/App.vue"],
    ["packages/Dep.ts", "src/App.vue"],
  );
  for (const [name, mutate, message] of [
    ["duplicate", (value: any) => value.requested.files.push("src/App.vue"), /duplicate file/],
    ["order", (value: any) => value.checked.files.reverse(), /files are not sorted/],
    ["traversal", (value: any) => (value.checked.files[0] = "../Dep.vue"), /normalized relative/],
    ["digest", (value: any) => (value.checked.sha256 = "0".repeat(64)), /sha256 is inconsistent/],
    ["count", (value: any) => (value.checked.fileCount = 1), /fileCount is inconsistent/],
    [
      "partition",
      (value: any) => {
        value.transitiveAuthored.files = [];
        value.transitiveAuthored.fileCount = 0;
        value.transitiveAuthored.sha256 = digest([]);
      },
      /classes do not partition checked files/,
    ],
  ] as const) {
    const mutated = structuredClone(valid);
    mutate(mutated);
    assert.throws(() => summarizeTypecheckerCoverage(mutated), message, name);
  }
});

test("fixture matrix writes exact raw and compact transitive coverage evidence", () => {
  const fixtureDir = fs.mkdtempSync(path.join(root, "tests", "_fixtures", "matrix-coverage-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-matrix-coverage-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-matrix-coverage-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, ".storybook"), { recursive: true });
  fs.mkdirSync(path.join(fixtureDir, ".yarn", "cache"), { recursive: true });
  fs.mkdirSync(path.join(fixtureDir, "node_modules", "pkg"), { recursive: true });
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.mkdirSync(path.join(fixtureDir, "packages"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, ".storybook", "Hidden.ts"), "export {};\n");
  fs.writeFileSync(path.join(fixtureDir, ".yarn", "cache", "Injected.ts"), "export {};\n");
  fs.writeFileSync(path.join(fixtureDir, "node_modules", "pkg", "Injected.ts"), "export {};\n");
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(path.join(fixtureDir, "packages", "Dep.ts"), "export {};\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node\nprocess.stdout.write(JSON.stringify(${JSON.stringify(
      output([".storybook/Hidden.ts", "packages/Dep.ts", "src/App.vue"]),
    )}));\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "coverage-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["src/**/*.vue"],
      },
      "typechecker",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "ok");
    assert.deepEqual(run.coverage, {
      requestedFileCount: 1,
      requestedSha256: digest(["src/App.vue"]),
      transitiveAuthoredFileCount: 2,
      transitiveAuthoredSha256: digest([".storybook/Hidden.ts", "packages/Dep.ts"]),
      checkedFileCount: 3,
      checkedSha256: digest([".storybook/Hidden.ts", "packages/Dep.ts", "src/App.vue"]),
    });
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.deepEqual(raw.typecheckerCoverage.checked.files, [
      ".storybook/Hidden.ts",
      "packages/Dep.ts",
      "src/App.vue",
    ]);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});

test("fixture matrix isolates no-tsconfig typechecker projects from ancestor discovery", () => {
  const fixtureDir = fs.mkdtempSync(path.join(root, "tests", "_fixtures", "matrix-no-tsconfig-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-matrix-no-tsconfig-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-matrix-no-tsconfig-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  const invocationPath = path.join(fakeDir, "invocation.json");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
const args = process.argv.slice(2);
const tsconfigIndex = args.indexOf("--tsconfig");
const tsconfig = tsconfigIndex === -1 ? null : args[tsconfigIndex + 1];
const source = tsconfig == null ? null : fs.readFileSync(path.resolve(process.cwd(), tsconfig), "utf8");
fs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ args, tsconfig, source }));
const output =
  tsconfig == null
    ? {
        files: [{ file: "src/App.vue", diagnostics: [] }],
        programs: [
          {
            root: ".",
            files: [
              "src/App.vue",
              path.resolve(process.cwd(), "../../node_modules/@types/node/assert.d.ts"),
            ],
          },
        ],
        errorCount: 0,
        warningCount: 0,
        fileCount: 1,
      }
    : {
        files: [{ file: "src/App.vue", diagnostics: [] }],
        programs: [
          { root: ".", tsconfig, compilerOptions: {}, files: ["src/App.vue"] },
        ],
        errorCount: 0,
        warningCount: 0,
        fileCount: 1,
      };
process.stdout.write(JSON.stringify(output));\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "matrix-no-tsconfig",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["src/**/*.vue"],
      },
      "typechecker",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "ok", run.failure);
    assert.deepEqual(run.coverage, {
      requestedFileCount: 1,
      requestedSha256: digest(["src/App.vue"]),
      transitiveAuthoredFileCount: 0,
      transitiveAuthoredSha256: digest([]),
      checkedFileCount: 1,
      checkedSha256: digest(["src/App.vue"]),
    });
    const invocation = JSON.parse(fs.readFileSync(invocationPath, "utf8"));
    assert.equal(typeof invocation.tsconfig, "string");
    assert.notEqual(invocation.tsconfig, "");
    assert.deepEqual(JSON.parse(invocation.source), { compilerOptions: {} });
    assert.equal(fs.existsSync(path.join(fixtureDir, invocation.tsconfig)), false);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
