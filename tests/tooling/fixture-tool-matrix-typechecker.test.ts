import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { runTool } from "../../legacy-tools/fixtures/tool-matrix-run.mjs";
import { validateTypecheckerOutput } from "../../legacy-tools/fixtures/tool-matrix-typechecker.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function validOutput() {
  return {
    files: [
      {
        file: "src/App.vue",
        diagnostics: ["error:1:1 [TS1] synthetic error"],
      },
    ],
    programs: [
      {
        root: ".",
        files: ["src/App.vue"],
      },
    ],
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
  };
}

test("typechecker oracle accepts exact error, warning, project, and zero-file reports", () => {
  const errorOutput = validOutput();
  const before = structuredClone(errorOutput);
  validateTypecheckerOutput({ id: "errors" }, errorOutput, 1);
  assert.deepEqual(errorOutput, before, "validation must not mutate the report");

  const warningOutput = validOutput();
  warningOutput.files[0].diagnostics = ["warning:1:1 [TS2] synthetic warning"];
  warningOutput.errorCount = 0;
  warningOutput.warningCount = 1;
  validateTypecheckerOutput({ id: "warnings" }, warningOutput, 0);

  const projectOutput = validOutput();
  projectOutput.files.push({
    file: "tsconfig.json",
    diagnostics: ["error:1:1 [TS3] synthetic project error"],
  });
  projectOutput.errorCount = 2;
  validateTypecheckerOutput({ id: "project-error" }, projectOutput, 1);

  const byteOrderedOutput = validOutput();
  byteOrderedOutput.files = [
    { file: "src/B.vue", diagnostics: ["error:1:1 [TS1] uppercase"] },
    { file: "src/a.vue", diagnostics: ["error:1:1 [TS1] lowercase"] },
  ];
  byteOrderedOutput.fileCount = 2;
  byteOrderedOutput.errorCount = 2;
  validateTypecheckerOutput({ id: "byte-order" }, byteOrderedOutput, 1);

  validateTypecheckerOutput(
    { id: "no-sfc", expectedVueFileCount: 0 },
    { files: [], programs: [], errorCount: 0, warningCount: 0, fileCount: 0 },
    0,
    [],
  );
});

test("typechecker oracle requires every fixture input and rejects substitutions", () => {
  const output = validOutput();
  output.files.push({ file: "src/Card.vue", diagnostics: [] });
  output.fileCount = 2;

  validateTypecheckerOutput({ id: "complete" }, output, 1, ["src/App.vue", "src/Card.vue"]);

  assert.throws(
    () =>
      validateTypecheckerOutput({ id: "partial" }, validOutput(), 1, [
        "src/App.vue",
        "src/Card.vue",
      ]),
    /checked files are missing requested fixture inputs: \[src\/Card\.vue\]/,
  );

  const substituted = validOutput();
  assert.throws(
    () => validateTypecheckerOutput({ id: "substituted" }, substituted, 1, ["src/Other.vue"]),
    /checked files are missing requested fixture inputs: \[src\/Other\.vue\]/,
  );
});

test("typechecker oracle rejects malformed or internally inconsistent reports", () => {
  const cases = [
    {
      name: "null envelope",
      output: null,
      message: /envelope must be an object/,
    },
    {
      name: "extra envelope key",
      mutate: (output: any) => (output.extra = true),
      message: /envelope keys must be/,
    },
    {
      name: "negative counter",
      mutate: (output: any) => (output.errorCount = -1),
      message: /errorCount must be a non-negative safe integer/,
    },
    {
      name: "fractional counter",
      mutate: (output: any) => (output.fileCount = 0.5),
      message: /fileCount must be a non-negative safe integer/,
    },
    {
      name: "non-array files",
      mutate: (output: any) => (output.files = {}),
      message: /files must be an array/,
    },
    {
      name: "excess file count",
      mutate: (output: any) => (output.fileCount = 2),
      message: /fileCount 2 exceeds 1 file entries/,
    },
    {
      name: "non-empty fixture with no checked files",
      output: { files: [], programs: [], errorCount: 0, warningCount: 0, fileCount: 0 },
      exitCode: 0,
      message: /non-empty fixture checked zero Vue files/,
    },
    {
      name: "declared zero fixture with a checked file",
      project: { id: "zero", expectedVueFileCount: 0 },
      message: /expected zero checked files, received 1/,
    },
    {
      name: "extra file key",
      mutate: (output: any) => (output.files[0].extra = true),
      message: /files\[0\] keys must be/,
    },
    {
      name: "absolute Unix path",
      mutate: (output: any) => (output.files[0].file = "/src/App.vue"),
      message: /must be a normalized relative path/,
    },
    {
      name: "absolute Windows path",
      mutate: (output: any) => (output.files[0].file = "C:\\src\\App.vue"),
      message: /must be a normalized relative path/,
    },
    {
      name: "parent traversal",
      mutate: (output: any) => (output.files[0].file = "../src/App.vue"),
      message: /must be a normalized relative path/,
    },
    {
      name: "duplicate file",
      mutate: (output: any) => {
        output.files.push(structuredClone(output.files[0]));
        output.fileCount = 2;
        output.errorCount = 2;
      },
      message: /duplicate file entry/,
    },
    {
      name: "non-Vue checked file",
      mutate: (output: any) => (output.files[0].file = "src/App.ts"),
      message: /checked file is not a Vue SFC/,
    },
    {
      name: "unsorted checked files",
      mutate: (output: any) => {
        output.files = [
          { file: "src/a.vue", diagnostics: ["error:1:1 [TS1] lowercase"] },
          { file: "src/B.vue", diagnostics: ["error:1:1 [TS1] uppercase"] },
        ];
        output.fileCount = 2;
        output.errorCount = 2;
      },
      message: /checked file entries are not sorted/,
    },
    {
      name: "non-array diagnostics",
      mutate: (output: any) => (output.files[0].diagnostics = {}),
      message: /diagnostics must be an array/,
    },
    {
      name: "empty diagnostic",
      mutate: (output: any) => (output.files[0].diagnostics = [""]),
      message: /must be a non-empty string/,
    },
    {
      name: "unknown diagnostic prefix",
      mutate: (output: any) => (output.files[0].diagnostics = ["info: synthetic"]),
      message: /has no error or warning prefix/,
    },
    {
      name: "empty project-level entry",
      mutate: (output: any) => output.files.push({ file: "tsconfig.json", diagnostics: [] }),
      message: /project-level file entry has no diagnostics/,
    },
    {
      name: "error count mismatch",
      mutate: (output: any) => (output.errorCount = 0),
      message: /errorCount 0 does not match 1 diagnostics/,
    },
    {
      name: "warning count mismatch",
      mutate: (output: any) => {
        output.files[0].diagnostics = ["warning:1:1 [TS2] warning"];
        output.errorCount = 0;
      },
      message: /warningCount 0 does not match 1 diagnostics/,
    },
    {
      name: "exit code mismatch",
      exitCode: 0,
      message: /exit code 0 does not match expected 1/,
    },
  ];

  for (const fixtureCase of cases) {
    const output = "output" in fixtureCase ? fixtureCase.output : validOutput();
    fixtureCase.mutate?.(output);
    assert.throws(
      () =>
        validateTypecheckerOutput(
          fixtureCase.project ?? { id: "fixture" },
          output,
          fixtureCase.exitCode ?? 1,
        ),
      fixtureCase.message,
      fixtureCase.name,
    );
  }
});

test("fixture tool matrix records typechecker schema failures", () => {
  const fixtureDir = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "tool-matrix-typechecker-"),
  );
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typechecker-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typechecker-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node
process.stdout.write(JSON.stringify({ files: [], programs: [], errorCount: 0, warningCount: 0, fileCount: 0 }));\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "typechecker-schema-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["**/*.vue"],
      },
      "typechecker",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "failed");
    assert.match(run.failure, /non-empty fixture checked zero Vue files/);
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.match(raw.validationError, /non-empty fixture checked zero Vue files/);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects partial typechecker coverage", () => {
  const fixtureDir = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "tool-matrix-typechecker-partial-"),
  );
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typechecker-partial-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typechecker-partial-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(path.join(fixtureDir, "src", "Card.vue"), "<template />\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node
process.stdout.write(JSON.stringify({ files: [{ file: "src/App.vue", diagnostics: [] }], programs: [{ root: ".", files: ["src/App.vue"] }], errorCount: 0, warningCount: 0, fileCount: 1 }));\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "typechecker-partial-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["**/*.vue"],
      },
      "typechecker",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "failed");
    assert.match(
      run.failure,
      /checked files are missing requested fixture inputs: \[src\/Card\.vue\]/,
    );
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.match(
      raw.validationError,
      /checked files are missing requested fixture inputs: \[src\/Card\.vue\]/,
    );
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
