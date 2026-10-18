import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import {
  resolveBaselineRuntime,
  runBaseline,
  selectComparableRules,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";
import { collectBaselineFindings } from "../../tools/support/compat/fixtures/lint-divergence-records.mjs";
import { evaluateBudget } from "../../tools/support/compat/fixtures/lint-divergence-budget.mjs";
import { baselineScript as candidate } from "../../tools/support/fixtures/lint_divergence/runtime.ts";

const root = path.resolve(import.meta.dirname, "../..");
const corpus = path.join(root, "tests/_fixtures/differential/lint/reference-script-language");
const originals = JSON.parse(fs.readFileSync(path.join(corpus, "originals.json"), "utf8"));
const frozen = fs.readFileSync(path.join(corpus, "before-script.mjs.txt"), "utf8");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.ok(candidate);
assert.equal(digest(frozen), originals.beforeScriptSha256);
assert.equal(originals.inputs.length, 8);
assert.equal(new Set(originals.inputs.map((input) => input.project)).size, 6);
const runtime = resolveBaselineRuntime();
const rules = selectComparableRules(
  JSON.parse(fs.readFileSync(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json"))),
  "ecosystem",
  { includeUnimplemented: true },
).rules;
const transport = (value) => JSON.parse(JSON.stringify(value));

function execute(script, cwd, filename, selected = rules) {
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
    cwd: root,
    input: JSON.stringify({
      benchPackageJson: path.join(root, "tools/benchmarks/scripts/package.json"),
      cwd,
      files: [filename],
      rules: selected,
    }),
    encoding: "utf8",
    timeout: 30000,
  });
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  return JSON.parse(result.stdout).results;
}

async function publicReference(cwd, filename, jsx) {
  // Independently configure the pinned public provider's documented JSX mode.
  const eslint = new runtime.ESLint({
    cwd,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: runtime.vueParser,
          parserOptions: {
            parser: runtime.scriptParser,
            ecmaVersion: "latest",
            sourceType: "module",
            ecmaFeatures: { jsx },
            extraFileExtensions: [".vue"],
          },
        },
        linterOptions: { reportUnusedDisableDirectives: "off" },
        plugins: { vue: runtime.plugin },
        processor: "vue/vue",
        rules,
      },
    ],
  });
  return transport(await eslint.lintFiles([filename]));
}

function strictVerdict(results, cwd) {
  const classified = collectBaselineFindings(results, cwd);
  const budget = evaluateBudget({
    project: "script-language-control",
    files: { comparedCount: 1 },
    baseline: { comparedRuleCount: Object.keys(rules).length },
    divergence: {
      summary: {
        falsePositiveCount: 0,
        falseNegativeCount: 0,
        baselineParseErrorCount: classified.parseErrorCount,
        baselineInvalidRangeCount: classified.invalidRangeCount,
      },
    },
  });
  assert.equal(budget.maxFalsePositiveCount, 0);
  assert.equal(budget.maxFalseNegativeCount, 0);
  return budget;
}

for (const original of originals.inputs) {
  await test(`whole pinned ${original.project}/${original.path} retains its reference verdict`, async () => {
    assert.equal(digest(original.source), original.sourceSha256);
    const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reference-language-"));
    const filename = "Original.vue";
    try {
      fs.writeFileSync(path.join(cwd, filename), original.source);
      const before = execute(frozen, cwd, filename, originals.originalRules);
      assert.deepEqual(
        before,
        original.before.results.map((result) => ({
          ...result,
          filePath: path.join(cwd, filename),
        })),
        "complete replayed fatal packet, source, and provider metadata",
      );
      assert.equal(strictVerdict(before, cwd).verdict, "unusable");
      const expected = await publicReference(cwd, filename, !original.expectedRepaired);
      const measured = [
        execute(candidate, cwd, filename),
        transport((await runBaseline(runtime, cwd, [filename], rules)).results),
      ];
      for (const results of measured) {
        assert.deepEqual(results, expected, "complete independent public ESLint result");
        const budget = strictVerdict(results, cwd);
        assert.equal(budget.verdict, original.expectedRepaired ? "passed" : "unusable");
        assert.equal(budget.passed, original.expectedRepaired);
        if (!original.expectedRepaired)
          assert.deepEqual(results, before, "unsupported source remains an exact strict refusal");
      }
    } finally {
      fs.rmSync(cwd, { recursive: true, force: true });
    }
  });
}

for (const lang of ["jsx", "tsx"]) {
  await test(`${lang} retains actual JSX parsing and unsuppressed Vue findings`, async () => {
    const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reference-jsx-"));
    try {
      const body = `<script setup lang="${lang}">const render = () => <div />;</script>\n<template><div v-html="html" /></template>\n`;
      fs.writeFileSync(path.join(cwd, "Control.vue"), body);
      const expected = await publicReference(cwd, "Control.vue", true);
      assert.equal(expected[0].fatalErrorCount, 0);
      assert.equal(
        expected[0].messages.filter((message) => message.ruleId === "vue/no-v-html").length,
        1,
      );
      for (const results of [
        execute(candidate, cwd, "Control.vue"),
        transport((await runBaseline(runtime, cwd, ["Control.vue"], rules)).results),
      ])
        assert.deepEqual(results, expected);
      assert.deepEqual(execute(frozen, cwd, "Control.vue"), expected);
    } finally {
      fs.rmSync(cwd, { recursive: true, force: true });
    }
  });
}

await test("genuinely malformed TypeScript remains a complete strict fatal refusal", async () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reference-malformed-"));
  try {
    fs.writeFileSync(
      path.join(cwd, "Malformed.vue"),
      '<script setup lang="ts">const broken = ;</script>\n',
    );
    const expected = await publicReference(cwd, "Malformed.vue", false);
    assert.equal(expected[0].fatalErrorCount, 1);
    for (const results of [
      execute(candidate, cwd, "Malformed.vue"),
      transport((await runBaseline(runtime, cwd, ["Malformed.vue"], rules)).results),
    ]) {
      assert.deepEqual(results, expected);
      assert.equal(strictVerdict(results, cwd).verdict, "unusable");
    }
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});
