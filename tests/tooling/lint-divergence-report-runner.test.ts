import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");

test("the TypeScript runner counts the same ignore-filtered files as vize lint", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-ignore-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-ignore-report-"));
  try {
    fs.mkdirSync(path.join(fixtureDir, "generated"));
    fs.writeFileSync(path.join(fixtureDir, ".gitignore"), "generated/\n");
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      ["<template>", "  <div />", "</template>", ""].join("\n"),
    );
    fs.writeFileSync(
      path.join(fixtureDir, "generated", "Ignored.vue"),
      ["<template>", "  <section />", "</template>", ""].join("\n"),
    );
    const fakeVize = path.join(fixtureDir, "fake-vize.mjs");
    fs.writeFileSync(
      fakeVize,
      [
        "#!/usr/bin/env node",
        "if (process.argv[2] !== 'lint') process.exit(2);",
        'process.stdout.write(JSON.stringify([{ file: "App.vue", messages: [] }]));',
        "",
      ].join("\n"),
    );
    fs.chmodSync(fakeVize, 0o755);
    const registryPath = path.join(fixtureDir, "registry.json");
    fs.writeFileSync(
      registryPath,
      JSON.stringify({
        projects: [
          {
            id: "lint-ignore-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["**/*.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const result = spawnSync(
      process.execPath,
      [
        "tools/commands/fixtures/lint-divergence-report.ts",
        "--registry",
        registryPath,
        "--output-dir",
        outputDir,
        "--vize-bin",
        fakeVize,
        "--preset",
        "no-rules",
        "--budget-mode",
        "record-only",
        "--timeout-ms",
        "30000",
      ],
      { cwd: repoRoot, encoding: "utf8", env: { ...process.env, LANG: "C", LC_ALL: "C" } },
    );

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const artifact = JSON.parse(
      fs.readFileSync(path.join(outputDir, "lint-ignore-fixture-lint-divergence.json"), "utf8"),
    );
    assert.equal(artifact.files.comparedCount, 1);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("the TypeScript runner pairs prefixed unused-component location divergences", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-unused-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-unused-report-"));
  try {
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      [
        "<template>",
        "  <main />",
        "</template>",
        "<script>",
        "import UploaderFile from './UploaderFile.vue';",
        "import UploaderFiles from './UploaderFiles.vue';",
        "export default {",
        "  components: {",
        "    UploaderFile,",
        "    UploaderFiles,",
        "  },",
        "};",
        "</script>",
        "",
      ].join("\n"),
    );
    const fakeVize = path.join(fixtureDir, "fake-vize.mjs");
    fs.writeFileSync(
      fakeVize,
      [
        "#!/usr/bin/env node",
        "if (process.argv[2] !== 'lint') process.exit(2);",
        "process.stdout.write(JSON.stringify([{",
        '  file: "App.vue",',
        "  messages: [",
        "    {",
        '      ruleId: "vue/no-unused-components",',
        '      severity: "warning",',
        "      line: 1, column: 11, endLine: 2, endColumn: 12,",
        "      message: \"[vize:vue/no-unused-components] Component 'UploaderFile' is registered but never used in template\",",
        "    },",
        "    {",
        '      ruleId: "vue/no-unused-components",',
        '      severity: "warning",',
        "      line: 1, column: 11, endLine: 2, endColumn: 12,",
        "      message: \"[vize:vue/no-unused-components] Component 'UploaderFiles' is registered but never used in template\",",
        "    },",
        "  ],",
        "}]));",
        "",
      ].join("\n"),
    );
    fs.chmodSync(fakeVize, 0o755);
    const registryPath = path.join(fixtureDir, "registry.json");
    fs.writeFileSync(
      registryPath,
      JSON.stringify({
        projects: [
          {
            id: "lint-unused-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["App.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const result = spawnSync(
      process.execPath,
      [
        "tools/commands/fixtures/lint-divergence-report.ts",
        "--registry",
        registryPath,
        "--output-dir",
        outputDir,
        "--vize-bin",
        fakeVize,
        "--budget-mode",
        "enforce",
        "--timeout-ms",
        "30000",
      ],
      { cwd: repoRoot, encoding: "utf8", env: { ...process.env, LANG: "C", LC_ALL: "C" } },
    );

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const artifact = JSON.parse(
      fs.readFileSync(path.join(outputDir, "lint-unused-fixture-lint-divergence.json"), "utf8"),
    );
    assert.equal(artifact.divergence.summary.falsePositiveCount, 0);
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    assert.equal(artifact.divergence.summary.ruleLocationDivergenceCount, 2);
    assert.deepEqual(
      artifact.divergence.ruleLocationDivergences.map(
        (entry: { subject: string }) => entry.subject,
      ),
      ["component:UploaderFile", "component:UploaderFiles"],
    );
    assert.equal(artifact.budget.passed, true);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("the TypeScript runner keeps embedded brackets in unused-component messages", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-unused-bracket-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-unused-bracket-report-"));
  try {
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      [
        "<template>",
        "  <main />",
        "</template>",
        "<script>",
        "import UploaderFile from './UploaderFile.vue';",
        "export default {",
        "  components: {",
        "    UploaderFile,",
        "  },",
        "};",
        "</script>",
        "",
      ].join("\n"),
    );
    const fakeVize = path.join(fixtureDir, "fake-vize.mjs");
    fs.writeFileSync(
      fakeVize,
      [
        "#!/usr/bin/env node",
        "if (process.argv[2] !== 'lint') process.exit(2);",
        "process.stdout.write(JSON.stringify([{",
        '  file: "App.vue",',
        "  messages: [",
        "    {",
        '      ruleId: "vue/no-unused-components",',
        '      severity: "warning",',
        "      line: 1, column: 11, endLine: 2, endColumn: 12,",
        "      message: \"debug ] Component 'UploaderFile' is registered but never used in template\",",
        "    },",
        "  ],",
        "}]));",
        "",
      ].join("\n"),
    );
    fs.chmodSync(fakeVize, 0o755);
    const registryPath = path.join(fixtureDir, "registry.json");
    fs.writeFileSync(
      registryPath,
      JSON.stringify({
        projects: [
          {
            id: "lint-unused-bracket-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["App.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const result = spawnSync(
      process.execPath,
      [
        "tools/commands/fixtures/lint-divergence-report.ts",
        "--registry",
        registryPath,
        "--output-dir",
        outputDir,
        "--vize-bin",
        fakeVize,
        "--budget-mode",
        "record-only",
        "--timeout-ms",
        "30000",
      ],
      { cwd: repoRoot, encoding: "utf8", env: { ...process.env, LANG: "C", LC_ALL: "C" } },
    );

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const artifact = JSON.parse(
      fs.readFileSync(
        path.join(outputDir, "lint-unused-bracket-fixture-lint-divergence.json"),
        "utf8",
      ),
    );
    assert.equal(artifact.divergence.summary.falsePositiveCount, 1);
    assert.equal(artifact.divergence.summary.falseNegativeCount, 1);
    assert.equal(artifact.divergence.summary.ruleLocationDivergenceCount, 0);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
