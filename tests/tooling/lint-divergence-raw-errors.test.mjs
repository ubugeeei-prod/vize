import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  resolveBaselineRuntime,
  runBaseline,
  selectComparableRules,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

const root = path.resolve(import.meta.dirname, "../..");

await test("the actual TypeScript report preserves complete raw fatal results before strict classification", async () => {
  fs.mkdirSync(path.join(root, "target"), { recursive: true });
  const cwd = fs.mkdtempSync(path.join(root, "target", "lint-fatal-custody-"));
  try {
    const source = '<script setup lang="ts">const broken = ;</script>\n';
    fs.writeFileSync(path.join(cwd, "Malformed.vue"), source);
    const executable = path.join(cwd, "fake-vize.mjs");
    fs.writeFileSync(
      executable,
      '#!/usr/bin/env node\nprocess.stdout.write(JSON.stringify([{ file: "Malformed.vue", messages: [] }]));\n',
    );
    fs.chmodSync(executable, 0o755);
    const registry = path.join(cwd, "registry.json");
    fs.writeFileSync(
      registry,
      JSON.stringify({
        projects: [
          {
            id: "fatal-custody",
            revision: "0".repeat(40),
            fixturePath: path.relative(root, cwd),
            vueGlobs: ["Malformed.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );
    const output = path.join(cwd, "report");
    const processResult = spawnSync(
      process.execPath,
      [
        "tools/commands/fixtures/lint-divergence-report.ts",
        "--registry",
        registry,
        "--output-dir",
        output,
        "--vize-bin",
        executable,
        "--preset",
        "ecosystem",
        "--budget-mode",
        "enforce",
      ],
      { cwd: root, encoding: "utf8", timeout: 120000 },
    );
    assert.equal(processResult.status, 1, `${processResult.stdout}\n${processResult.stderr}`);
    const report = JSON.parse(
      fs.readFileSync(path.join(output, "fatal-custody-lint-divergence.json")),
    );
    const rules = selectComparableRules(
      JSON.parse(
        fs.readFileSync(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json")),
      ),
      "ecosystem",
    ).rules;
    const expected = JSON.parse(
      JSON.stringify(
        (await runBaseline(resolveBaselineRuntime(), cwd, ["Malformed.vue"], rules)).results,
      ),
    );
    assert.deepEqual(report.baseline.parseErrorResults, expected);
    assert.equal(report.baseline.parseErrorResults[0].source, source);
    assert.equal(report.baseline.parseErrorResults[0].fatalErrorCount, 1);
    assert.equal(report.divergence.summary.baselineParseErrorCount, 1);
    assert.equal(report.budget.verdict, "unusable");
    assert.equal(report.budget.passed, false);
    assert.equal(report.budget.maxFalsePositiveCount, 0);
    assert.equal(report.budget.maxFalseNegativeCount, 0);
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});
