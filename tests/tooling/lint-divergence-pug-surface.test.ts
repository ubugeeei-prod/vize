// Davinci P4-12c: Patina lints `<template lang="pug">` through the pug
// dialect, but the ESLint baseline (vue-eslint-parser) cannot parse pug, so
// the lint-divergence runner reports Patina findings inside a pug template
// body as outside the comparable surface instead of scoring them as false
// positives. Findings elsewhere in the same SFC are still compared.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");

test("the Rust runner keeps pug template findings out of the scored surface", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-pug-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-pug-report-"));
  try {
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      [
        '<template lang="pug">',
        "ul",
        '  li(v-for="item in items") {{ item }}',
        "</template>",
        "<script>",
        "export default { data: () => ({ items: [1] }) };",
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
        "  messages: [{",
        '    ruleId: "vue/require-v-for-key",',
        '    severity: "error",',
        "    line: 3, column: 6, endLine: 3, endColumn: 27,",
        "    message: \"[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives.\",",
        "  }],",
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
            id: "lint-pug-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["App.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const result = spawnSync(
      "rust-script",
      [
        "tools/commands/fixtures/lint-divergence-report.rs",
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
      fs.readFileSync(path.join(outputDir, "lint-pug-fixture-lint-divergence.json"), "utf8"),
    );
    const summary = artifact.divergence.summary;
    assert.equal(summary.falsePositiveCount, 0);
    assert.equal(summary.patinaOutsideBaselineSurfaceCount, 1);
    assert.equal(summary.patinaFindingCount, 1);
    assert.deepEqual(
      artifact.divergence.patinaOutsideBaselineSurface.map(
        (entry: { ruleId: string; line: number }) => [entry.ruleId, entry.line],
      ),
      [["vue/require-v-for-key", 3]],
    );
    assert.equal(artifact.budget.passed, true);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
