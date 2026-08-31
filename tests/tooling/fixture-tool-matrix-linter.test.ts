import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { runTool } from "../../legacy-tools/fixtures/tool-matrix-run.mjs";
import { validateLinterOutput } from "../../legacy-tools/fixtures/tool-matrix-linter.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function diagnostic(severity = 2) {
  return {
    ruleId: "vue/synthetic-rule",
    ruleDocsPath: "rules/vue/synthetic-rule.md",
    severity,
    message: "synthetic diagnostic",
    line: 1,
    column: 2,
    endLine: 1,
    endColumn: 3,
  };
}

function validOutput() {
  return [
    {
      file: "src/App.vue",
      messages: [diagnostic(2), { ...diagnostic(1), help: "synthetic help" }],
      errorCount: 1,
      warningCount: 1,
    },
  ];
}

test("linter oracle accepts exact error, warning, clean, and zero-file reports", () => {
  const output = validOutput();
  const before = structuredClone(output);
  validateLinterOutput({ id: "mixed" }, output, 1);
  assert.deepEqual(output, before, "validation must not mutate the report");

  const warningOnly = validOutput();
  warningOnly[0].messages = [diagnostic(1)];
  warningOnly[0].errorCount = 0;
  validateLinterOutput({ id: "warnings" }, warningOnly, 0);

  const clean = validOutput();
  clean[0].messages = [];
  clean[0].errorCount = 0;
  clean[0].warningCount = 0;
  validateLinterOutput({ id: "clean" }, clean, 0);

  validateLinterOutput({ id: "no-sfc", expectedVueFileCount: 0 }, [], 0, []);
});

test("linter oracle requires every fixture input and rejects substitutions", () => {
  const output = validOutput();
  output.push({
    file: "src/Card.vue",
    messages: [],
    errorCount: 0,
    warningCount: 0,
  });

  validateLinterOutput({ id: "complete" }, output, 1, ["src/App.vue", "src/Card.vue"]);

  assert.throws(
    () =>
      validateLinterOutput({ id: "partial" }, validOutput(), 1, ["src/App.vue", "src/Card.vue"]),
    /checked file count 1 does not match 2 inputs/,
  );

  assert.throws(
    () => validateLinterOutput({ id: "substituted" }, validOutput(), 1, ["src/Other.vue"]),
    /checked files do not match inputs: missing \[src\/Other\.vue\], unexpected \[src\/App\.vue\]/,
  );
});

test("linter oracle rejects malformed or internally inconsistent reports", () => {
  const cases = [
    {
      name: "non-array envelope",
      output: {},
      message: /envelope must be an array/,
    },
    {
      name: "non-empty fixture with no files",
      output: [],
      exitCode: 0,
      message: /non-empty fixture linted zero Vue files/,
    },
    {
      name: "declared zero fixture with a file",
      project: { id: "zero", expectedVueFileCount: 0 },
      message: /expected zero checked files, received 1/,
    },
    {
      name: "non-object file entry",
      mutate: (output: any) => (output[0] = null),
      message: /files\[0\] must be an object/,
    },
    {
      name: "extra file key",
      mutate: (output: any) => (output[0].extra = true),
      message: /files\[0\] keys must be/,
    },
    {
      name: "absolute Unix file path",
      mutate: (output: any) => (output[0].file = "/src/App.vue"),
      message: /must be a normalized relative path/,
    },
    {
      name: "absolute Windows file path",
      mutate: (output: any) => (output[0].file = "C:\\src\\App.vue"),
      message: /must be a normalized relative path/,
    },
    {
      name: "file parent traversal",
      mutate: (output: any) => (output[0].file = "../src/App.vue"),
      message: /must be a normalized relative path/,
    },
    {
      name: "non-Vue checked file",
      mutate: (output: any) => (output[0].file = "src/App.ts"),
      message: /checked file is not a Vue SFC/,
    },
    {
      name: "duplicate file",
      mutate: (output: any) => output.push(structuredClone(output[0])),
      message: /duplicate file entry/,
    },
    {
      name: "negative file counter",
      mutate: (output: any) => (output[0].errorCount = -1),
      message: /errorCount must be a non-negative safe integer/,
    },
    {
      name: "fractional file counter",
      mutate: (output: any) => (output[0].warningCount = 0.5),
      message: /warningCount must be a non-negative safe integer/,
    },
    {
      name: "non-array messages",
      mutate: (output: any) => (output[0].messages = {}),
      message: /messages must be an array/,
    },
    {
      name: "non-object message",
      mutate: (output: any) => (output[0].messages[0] = null),
      message: /messages\[0\] must be an object/,
    },
    {
      name: "extra message key",
      mutate: (output: any) => (output[0].messages[0].extra = true),
      message: /messages\[0\] keys must be/,
    },
    {
      name: "empty rule id",
      mutate: (output: any) => (output[0].messages[0].ruleId = ""),
      message: /ruleId must be non-empty/,
    },
    {
      name: "invalid docs path",
      mutate: (output: any) => (output[0].messages[0].ruleDocsPath = "../rule.md"),
      message: /ruleDocsPath must be a normalized relative path/,
    },
    {
      name: "empty message",
      mutate: (output: any) => (output[0].messages[0].message = ""),
      message: /message must be non-empty/,
    },
    {
      name: "empty help",
      mutate: (output: any) => (output[0].messages[1].help = ""),
      message: /help must be non-empty/,
    },
    {
      name: "invalid severity",
      mutate: (output: any) => (output[0].messages[0].severity = 3),
      message: /severity must be 1 or 2/,
    },
    {
      name: "zero location",
      mutate: (output: any) => (output[0].messages[0].line = 0),
      message: /line must be a positive safe integer/,
    },
    {
      name: "fractional location",
      mutate: (output: any) => (output[0].messages[0].column = 1.5),
      message: /column must be a positive safe integer/,
    },
    {
      name: "inverted line range",
      mutate: (output: any) => {
        output[0].messages[0].line = 2;
        output[0].messages[0].endLine = 1;
      },
      message: /inverted source range/,
    },
    {
      name: "inverted column range",
      mutate: (output: any) => {
        output[0].messages[0].column = 4;
        output[0].messages[0].endColumn = 3;
      },
      message: /inverted source range/,
    },
    {
      name: "error count mismatch",
      mutate: (output: any) => (output[0].errorCount = 0),
      message: /errorCount 0 does not match 1 messages/,
    },
    {
      name: "warning count mismatch",
      mutate: (output: any) => (output[0].warningCount = 0),
      message: /warningCount 0 does not match 1 messages/,
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
        validateLinterOutput(
          fixtureCase.project ?? { id: "fixture" },
          output,
          fixtureCase.exitCode ?? 1,
        ),
      fixtureCase.message,
      fixtureCase.name,
    );
  }
});

test("fixture tool matrix records linter schema failures", () => {
  const fixtureDir = fs.mkdtempSync(path.join(root, "tests", "_fixtures", "tool-matrix-linter-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(executable, '#!/usr/bin/env node\nprocess.stdout.write("[]");\n');
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "linter-schema-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["**/*.vue"],
      },
      "linter",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "failed");
    assert.match(run.failure, /non-empty fixture linted zero Vue files/);
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.match(raw.validationError, /non-empty fixture linted zero Vue files/);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects partial linter coverage", () => {
  const fixtureDir = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "tool-matrix-linter-partial-"),
  );
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-partial-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-partial-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(path.join(fixtureDir, "src", "Card.vue"), "<template />\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node
process.stdout.write(JSON.stringify([{ file: "src/App.vue", messages: [], errorCount: 0, warningCount: 0 }]));\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "linter-partial-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["**/*.vue"],
      },
      "linter",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "failed");
    assert.match(run.failure, /checked file count 1 does not match 2 inputs/);
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.match(raw.validationError, /checked file count 1 does not match 2 inputs/);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
