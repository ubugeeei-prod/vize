import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  snapshotFormatterInputs,
  validateFormatterOutput,
} from "../../legacy-tools/fixtures/tool-matrix-formatter.mjs";
import { runTool } from "../../legacy-tools/fixtures/tool-matrix-run.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const noFilesReport =
  "No .vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown files found matching the patterns\n";

function changedReport() {
  return [
    "Found 2 file(s)",
    "Would reformat: src/App.vue",
    "Would reformat: ./src/Card.vue",
    "",
    "Checked 2 file(s)",
    "  2 file(s) would be reformatted",
    "",
  ].join("\n");
}

test("formatter oracle accepts changed, mixed, clean, and zero-file reports", () => {
  assert.deepEqual(
    validateFormatterOutput({ id: "changed" }, "", changedReport(), 1, "same", "same", [
      "src/App.vue",
      "src/Card.vue",
    ]),
    {
      checkedFileCount: 2,
      changedFileCount: 2,
      unchangedFileCount: 0,
      changedPathsSha256: "60e7f3109278640e86d41597937d4032f96264285f92113a525cd05c56456913",
    },
  );
  assert.deepEqual(
    validateFormatterOutput(
      { id: "mixed" },
      "",
      [
        "Found 2 file(s)",
        "Would reformat: src/App.vue",
        "",
        "Checked 2 file(s)",
        "  1 file(s) would be reformatted",
        "  1 file(s) already formatted",
        "",
      ].join("\n"),
      1,
      "same",
      "same",
      ["src/App.vue", "src/Card.vue"],
    ),
    {
      checkedFileCount: 2,
      changedFileCount: 1,
      unchangedFileCount: 1,
      changedPathsSha256: "6b96ff4f4fad70570fff95f5a53de5486ef276e93fbc672e245f832afa8902c4",
    },
  );
  assert.deepEqual(
    validateFormatterOutput(
      { id: "clean" },
      "",
      "Found 1 file(s)\n\nChecked 1 file(s)\n  1 file(s) already formatted\n",
      0,
      "same",
      "same",
      ["src/App.vue"],
    ),
    {
      checkedFileCount: 1,
      changedFileCount: 0,
      unchangedFileCount: 1,
      changedPathsSha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    },
  );
  assert.deepEqual(
    validateFormatterOutput(
      { id: "no-sfc", expectedVueFileCount: 0 },
      "",
      noFilesReport,
      1,
      "same",
      "same",
      [],
    ),
    {
      checkedFileCount: 0,
      changedFileCount: 0,
      unchangedFileCount: 0,
      changedPathsSha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    },
  );
});

test("formatter oracle rejects malformed, inconsistent, or mutating checks", () => {
  const cases = [
    {
      name: "stdout output",
      stdout: "unexpected",
      message: /stdout must be empty/,
    },
    {
      name: "working tree mutation",
      after: "changed",
      message: /modified its working tree or input metadata/,
    },
    {
      name: "missing final newline",
      stderr: changedReport().slice(0, -1),
      message: /stderr must end with a newline/,
    },
    {
      name: "zero fixture unexpected report",
      project: { id: "zero", expectedVueFileCount: 0 },
      stderr: changedReport(),
      message: /zero-file fixture emitted an unexpected report/,
    },
    {
      name: "zero fixture exit mismatch",
      project: { id: "zero", expectedVueFileCount: 0 },
      stderr: noFilesReport,
      exitCode: 0,
      message: /zero-file exit code 0 does not match expected 1/,
    },
    {
      name: "missing found count",
      stderr: changedReport().replace("Found 2 file(s)", "Found files"),
      message: /missing found count/,
    },
    {
      name: "unsafe found count",
      stderr: changedReport().replace("Found 2 file(s)", "Found 9007199254740992 file(s)"),
      message: /found count is not a safe integer/,
    },
    {
      name: "zero found count",
      stderr: "Found 0 file(s)\n\nChecked 0 file(s)\n",
      exitCode: 0,
      message: /non-empty fixture formatted zero files/,
    },
    {
      name: "partial fixture coverage",
      expectedFiles: ["src/App.vue", "src/Card.vue", "src/Other.vue"],
      message: /found count 2 does not match 3 inputs/,
    },
    {
      name: "changed path outside fixture inputs",
      expectedFiles: ["src/Card.vue", "src/Other.vue"],
      message: /changed files are not fixture inputs: src\/App\.vue/,
    },
    {
      name: "absolute Unix path",
      stderr: changedReport().replace("src/App.vue", "/src/App.vue"),
      message: /changed file must be a normalized relative path/,
    },
    {
      name: "absolute Windows path",
      stderr: changedReport().replace("src/App.vue", "C:\\src\\App.vue"),
      message: /changed file must be a normalized relative path/,
    },
    {
      name: "parent traversal",
      stderr: changedReport().replace("src/App.vue", "../src/App.vue"),
      message: /changed file must be a normalized relative path/,
    },
    {
      name: "non-Vue path",
      stderr: changedReport().replace("src/App.vue", "src/App.ts"),
      message: /changed file is not a Vue SFC/,
    },
    {
      name: "missing summary separator",
      stderr: changedReport().replace("./src/Card.vue\n\nChecked", "./src/Card.vue\nChecked"),
      message: /missing blank line before formatter summary/,
    },
    {
      name: "missing checked count",
      stderr: changedReport().replace("Checked 2 file(s)", "Checked files"),
      message: /missing checked count/,
    },
    {
      name: "unexpected report line",
      stderr: changedReport().replace("\n", "\nextra\n"),
      message: /missing blank line before formatter summary/,
    },
    {
      name: "duplicate changed path",
      stderr: changedReport().replace("./src/Card.vue", "src/App.vue"),
      message: /duplicate changed paths/,
    },
    {
      name: "changed count mismatch",
      stderr: changedReport().replace(
        "2 file(s) would be reformatted",
        "1 file(s) would be reformatted",
      ),
      message: /changed count 1 does not match 2 paths/,
    },
    {
      name: "found and checked mismatch",
      stderr: changedReport().replace("Checked 2 file(s)", "Checked 3 file(s)"),
      message: /file counts do not reconcile/,
    },
    {
      name: "missing unchanged summary",
      stderr: "Found 1 file(s)\n\nChecked 1 file(s)\n",
      exitCode: 0,
      message: /file counts do not reconcile/,
    },
    {
      name: "exit code mismatch",
      exitCode: 0,
      message: /exit code 0 does not match expected 1/,
    },
  ];

  for (const fixtureCase of cases) {
    assert.throws(
      () =>
        validateFormatterOutput(
          fixtureCase.project ?? { id: "fixture" },
          fixtureCase.stdout ?? "",
          fixtureCase.stderr ?? changedReport(),
          fixtureCase.exitCode ?? 1,
          "same",
          fixtureCase.after ?? "same",
          fixtureCase.expectedFiles ?? null,
        ),
      fixtureCase.message,
      fixtureCase.name,
    );
  }
});

test("formatter input snapshot is fixture-scoped and detects changed inputs", () => {
  const fixtureDir = fs.mkdtempSync(path.join(root, "tests", "_fixtures", "formatter-snapshot-"));
  const siblingDir = fs.mkdtempSync(path.join(root, "tests", "_fixtures", "formatter-snapshot-"));
  const sourcePath = path.join(fixtureDir, "App.vue");
  fs.writeFileSync(sourcePath, "<template />\n");
  try {
    const before = snapshotFormatterInputs(fixtureDir, ["**/*.vue"]);
    fs.writeFileSync(path.join(siblingDir, "Other.vue"), "<template><aside /></template>\n");
    assert.equal(snapshotFormatterInputs(fixtureDir, ["**/*.vue"]), before);

    fs.writeFileSync(sourcePath, "<template><main /></template>\n");
    const afterContent = snapshotFormatterInputs(fixtureDir, ["**/*.vue"]);
    assert.notEqual(afterContent, before);
    fs.writeFileSync(sourcePath, "<template />\n");
    fs.chmodSync(sourcePath, 0o744);
    const afterMetadata = snapshotFormatterInputs(fixtureDir, ["**/*.vue"]);
    assert.notEqual(afterMetadata, before);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(siblingDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix records formatter mutations", () => {
  const fixtureDir = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "tool-matrix-formatter-"),
  );
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-formatter-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-formatter-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node
import fs from "node:fs";
fs.writeFileSync("src/App.vue", "<template><main /></template>\\n");
process.stderr.write("Found 1 file(s)\\nWould reformat: src/App.vue\\n\\nChecked 1 file(s)\\n  1 file(s) would be reformatted\\n");
process.exit(1);\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "formatter-mutation-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["**/*.vue"],
      },
      "formatter",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "failed");
    assert.match(run.failure, /modified its working tree or input metadata/);
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.match(raw.validationError, /modified its working tree or input metadata/);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects partial formatter coverage", () => {
  const fixtureDir = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "tool-matrix-formatter-partial-"),
  );
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-formatter-partial-"));
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-formatter-partial-report-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
  fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(path.join(fixtureDir, "src", "Card.vue"), "<template />\n");
  fs.writeFileSync(
    executable,
    `#!/usr/bin/env node
process.stderr.write("Found 1 file(s)\\n\\nChecked 1 file(s)\\n  1 file(s) already formatted\\n");\n`,
  );
  fs.chmodSync(executable, 0o755);

  try {
    const run = runTool(
      {
        id: "formatter-partial-fixture",
        fixturePath: path.relative(root, fixtureDir),
        vueGlobs: ["**/*.vue"],
      },
      "formatter",
      { dryRun: false, timeoutMs: 5_000 },
      { command: executable, prefix: [], label: executable },
      reportDir,
    );
    assert.equal(run.status, "failed");
    assert.match(run.failure, /found count 1 does not match 2 inputs/);
    const raw = JSON.parse(fs.readFileSync(path.resolve(root, run.outputPath as string), "utf8"));
    assert.match(raw.validationError, /found count 1 does not match 2 inputs/);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
