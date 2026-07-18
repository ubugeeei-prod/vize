import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { runTool } from "../../tools/fixtures/tool-matrix-run.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "fixtures", "tool-matrix-report.mjs");

function run(args: string[]) {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-compiler-"));
  const result = spawnSync(process.execPath, [toolPath, ...args, "--output-dir", outputDir], {
    cwd: root,
    encoding: "utf8",
  });
  return { outputDir, result };
}

function writeFakeVize(directory: string, body: string) {
  const executable = path.join(directory, "fake-vize.mjs");
  fs.writeFileSync(executable, `#!/usr/bin/env node\n${body}\n`);
  fs.chmodSync(executable, 0o755);
  return executable;
}

function withSyntheticFixture(runTest: () => void) {
  const fixtureDir = path.join(root, "tests", "_fixtures", "_git", "vue-vben-admin");
  const fixtureExisted = fs.existsSync(fixtureDir);
  const syntheticDir = path.join(fixtureDir, "apps");
  const syntheticDirExisted = fs.existsSync(syntheticDir);
  const fixtureFile = path.join(syntheticDir, "vize-tool-matrix-test.vue");
  fs.mkdirSync(syntheticDir, { recursive: true });
  assert.equal(fs.existsSync(fixtureFile), false, fixtureFile);
  fs.writeFileSync(fixtureFile, "<template><main /></template>\n");
  try {
    runTest();
  } finally {
    fs.rmSync(fixtureFile, { force: true });
    if (!syntheticDirExisted) fs.rmdirSync(syntheticDir);
    if (!fixtureExisted) fs.rmdirSync(fixtureDir);
  }
}

test("fixture tool matrix does not allocate compiler output for a missing fixture", () => {
  const compilerTempEntries = () =>
    fs
      .readdirSync(os.tmpdir())
      .filter((entry) => entry.startsWith("vize-fixture-compiler-"))
      .sort((left, right) => left.localeCompare(right));
  const before = compilerTempEntries();
  const run = runTool(
    {
      id: "missing-compiler-fixture",
      fixturePath: "tests/_fixtures/_git/missing-compiler-fixture",
      vueGlobs: ["**/*.vue"],
    },
    "compiler",
    { dryRun: false, timeoutMs: 1_000 },
    { command: process.execPath, prefix: [], label: process.execPath },
    os.tmpdir(),
  );
  assert.equal(run.status, "missing-fixture");
  assert.deepEqual(compilerTempEntries(), before);
});

test("fixture tool matrix mirrors compiler roots when validating artifact paths", () => {
  const cases = [
    {
      name: "root-wide glob",
      vueGlobs: ["**/*.vue"],
      source: "src/nested/App.vue",
      extraDirectories: [],
      output: "src/nested/App.json",
    },
    {
      name: "multiple existing roots",
      vueGlobs: ["apps/**/*.vue", "packages/**/*.vue"],
      source: "apps/admin/App.vue",
      extraDirectories: ["packages"],
      output: "apps/admin/App.json",
    },
    {
      name: "missing roots are excluded",
      vueGlobs: ["apps/**/*.vue", "missing/**/*.vue"],
      source: "apps/admin/App.vue",
      extraDirectories: [],
      output: "admin/App.json",
    },
  ] as const;

  for (const fixtureCase of cases) {
    const fixtureDir = fs.mkdtempSync(
      path.join(root, "tests", "_fixtures", "tool-matrix-compiler-layout-"),
    );
    const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-layout-"));
    const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-report-"));
    try {
      for (const directory of fixtureCase.extraDirectories) {
        fs.mkdirSync(path.join(fixtureDir, directory), { recursive: true });
      }
      const sourcePath = path.join(fixtureDir, fixtureCase.source);
      fs.mkdirSync(path.dirname(sourcePath), { recursive: true });
      fs.writeFileSync(sourcePath, "<template><main /></template>\n");

      const executable = writeFakeVize(
        fakeDir,
        `import fs from "node:fs"; import path from "node:path";
const output = process.argv[process.argv.indexOf("--output") + 1];
const artifact = path.join(output, ${JSON.stringify(fixtureCase.output)});
fs.mkdirSync(path.dirname(artifact), { recursive: true });
fs.writeFileSync(artifact, JSON.stringify({ filename: ${JSON.stringify(path.basename(fixtureCase.source))}, code: "export default {}", css: null, errors: [], warnings: [], script_lang: "js", macro_artifacts: [] }) + "\\n");`,
      );
      const run = runTool(
        {
          id: `compiler-layout-${fixtureCase.name}`,
          fixturePath: path.relative(root, fixtureDir),
          vueGlobs: [...fixtureCase.vueGlobs],
        },
        "compiler",
        { dryRun: false, timeoutMs: 5_000 },
        { command: executable, prefix: [], label: executable },
        reportDir,
      );
      assert.equal(run.status, "ok", `${fixtureCase.name}: ${JSON.stringify(run)}`);
    } finally {
      fs.rmSync(fixtureDir, { recursive: true, force: true });
      fs.rmSync(fakeDir, { recursive: true, force: true });
      fs.rmSync(reportDir, { recursive: true, force: true });
    }
  }
});

test("fixture tool matrix compiles every matched Vue file into validated JSON artifacts", () => {
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-compiler-"));
  const executable = writeFakeVize(
    fakeDir,
    `import fs from "node:fs"; import path from "node:path";
if (process.argv[2] === "--version") process.exit(0);
const output = process.argv[process.argv.indexOf("--output") + 1];
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, "vize-tool-matrix-test.json"), JSON.stringify({ filename: "vize-tool-matrix-test.vue", code: "export default {}", css: null, errors: [], warnings: ["synthetic warning"], script_lang: "js", macro_artifacts: [] }) + "\\n");
process.stdout.write("Built: vize-tool-matrix-test.vue\\n");`,
  );
  try {
    withSyntheticFixture(() => {
      const { outputDir, result } = run([
        "--project",
        "vue-vben-admin",
        "--tool",
        "compiler",
        "--vize-bin",
        executable,
      ]);
      try {
        assert.equal(result.status, 0, result.stderr);
        const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
        assert.deepEqual(
          {
            failedRuns: report.summary.failedRuns,
            okRuns: report.summary.okRuns,
            runStatus: report.projects[0].runs[0].status,
          },
          { failedRuns: 0, okRuns: 1, runStatus: "ok" },
        );
        const rawPath = path.resolve(root, report.projects[0].runs[0].outputPath);
        const raw = JSON.parse(fs.readFileSync(rawPath, "utf8"));
        assert.deepEqual(
          {
            inputFileCount: raw.compilerArtifacts.inputFileCount,
            outputFileCount: raw.compilerArtifacts.outputFileCount,
            errorCount: raw.compilerArtifacts.errorCount,
            warningCount: raw.compilerArtifacts.warningCount,
          },
          { inputFileCount: 1, outputFileCount: 1, errorCount: 0, warningCount: 1 },
        );
        assert.equal(
          raw.compilerArtifacts.sha256,
          "706ebeac28056a83af4a93074d352014f1f80d8427a5d9dd135ffc6c6473a796",
        );
        assert.equal("parsed" in raw, false);
      } finally {
        fs.rmSync(outputDir, { recursive: true, force: true });
      }
    });
  } finally {
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects incomplete and malformed compiler artifacts", () => {
  const cases = [
    {
      name: "missing",
      body: `if (process.argv[2] === "--version") process.exit(0);`,
      message: /compiler artifact count mismatch: 1 inputs, 0 outputs/,
    },
    {
      name: "malformed",
      body: `import fs from "node:fs"; import path from "node:path";
if (process.argv[2] === "--version") process.exit(0);
const output = process.argv[process.argv.indexOf("--output") + 1];
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, "vize-tool-matrix-test.json"), "{}\\n");`,
      message: /invalid compiler artifact keys in vize-tool-matrix-test\.json/,
    },
    {
      name: "unexpected-path",
      body: `import fs from "node:fs"; import path from "node:path";
if (process.argv[2] === "--version") process.exit(0);
const output = process.argv[process.argv.indexOf("--output") + 1];
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, "unrelated.json"), JSON.stringify({ filename: "vize-tool-matrix-test.vue", code: "export default {}", css: null, errors: [], warnings: [], script_lang: "js", macro_artifacts: [] }));`,
      message:
        /compiler artifact path mismatch: missing \[vize-tool-matrix-test\.json\], unexpected \[unrelated\.json\]/,
    },
    {
      name: "filename-mismatch",
      body: `import fs from "node:fs"; import path from "node:path";
if (process.argv[2] === "--version") process.exit(0);
const output = process.argv[process.argv.indexOf("--output") + 1];
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, "vize-tool-matrix-test.json"), JSON.stringify({ filename: "other.vue", code: "export default {}", css: null, errors: [], warnings: [], script_lang: "js", macro_artifacts: [] }));`,
      message:
        /compiler filename mismatch in vize-tool-matrix-test\.json: expected vize-tool-matrix-test\.vue, received other\.vue/,
    },
  ] as const;

  for (const fixtureCase of cases) {
    const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), `vize-compiler-${fixtureCase.name}-`));
    const executable = writeFakeVize(fakeDir, fixtureCase.body);
    try {
      withSyntheticFixture(() => {
        const { outputDir, result } = run([
          "--project",
          "vue-vben-admin",
          "--tool",
          "compiler",
          "--vize-bin",
          executable,
        ]);
        try {
          assert.equal(result.status, 1, fixtureCase.name);
          const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
          assert.equal(report.summary.failedRuns, 1);
          assert.match(report.projects[0].runs[0].failure, fixtureCase.message);
          const rawPath = path.resolve(root, report.projects[0].runs[0].outputPath);
          const raw = JSON.parse(fs.readFileSync(rawPath, "utf8"));
          assert.match(raw.validationError, fixtureCase.message);
        } finally {
          fs.rmSync(outputDir, { recursive: true, force: true });
        }
      });
    } finally {
      fs.rmSync(fakeDir, { recursive: true, force: true });
    }
  }
});
