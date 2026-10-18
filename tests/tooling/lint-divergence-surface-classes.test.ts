// The lint divergence comparator separates two finding classes from parity
// failures. Vize keeps camelCase `aria*`/`data*` component props, which
// eslint-plugin-vue's `vue/attribute-hyphenation` reports; those baseline
// findings are intentional divergences with a reason. eslint-plugin-vue's
// `vue/no-unused-components` checks only the `components` option, so a Vize
// finding for an unused `<script setup>` import is outside the baseline's
// surface. Every other finding of both rules is still scored.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const source = [
  "<script setup>",
  'import UnusedPanel from "./UnusedPanel.vue";',
  'import UsedPanel from "./UsedPanel.vue";',
  "</script>",
  "",
  "<template>",
  '  <UsedPanel ariaLabel="Close" :dataKey="id" fontSize="1" />',
  "</template>",
  "",
].join("\n");

function range(line: number, text: string) {
  const column = source.split("\n")[line - 1].indexOf(text) + 1;
  assert.ok(column > 0, text);
  return { line, column, endLine: line, endColumn: column + text.length };
}

test("the TypeScript runner keeps aria/data props and unused setup imports out of the scored findings", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-surface-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-surface-report-"));
  try {
    fs.writeFileSync(path.join(fixtureDir, "SurfacePanel.vue"), source);
    const messages = [
      {
        ruleId: "vue/attribute-hyphenation",
        severity: "warning",
        ...range(7, 'fontSize="1"'),
        message: "[vize:vue/attribute-hyphenation] Attribute should be hyphenated",
      },
      {
        ruleId: "vue/no-unused-components",
        severity: "warning",
        ...range(2, "UnusedPanel"),
        message:
          "[vize:vue/no-unused-components] Component 'UnusedPanel' is registered but never used in template",
      },
    ];
    const fakeVize = path.join(fixtureDir, "fake-vize.mjs");
    fs.writeFileSync(
      fakeVize,
      [
        "#!/usr/bin/env node",
        "if (process.argv[2] !== 'lint') process.exit(2);",
        `process.stdout.write(JSON.stringify([{ file: "SurfacePanel.vue", messages: ${JSON.stringify(messages)} }]));`,
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
            id: "lint-surface-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["SurfacePanel.vue"],
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
    const { divergence } = JSON.parse(
      fs.readFileSync(path.join(outputDir, "lint-surface-fixture-lint-divergence.json"), "utf8"),
    );
    type Finding = { ruleId: string; line: number; column: number; reason?: string };
    const pick = (entries: Finding[], rule: string) =>
      entries.filter((entry) => entry.ruleId === rule).map((entry) => [entry.line, entry.column]);
    const hyphenation = "vue/attribute-hyphenation";
    assert.deepEqual(pick(divergence.intentionalDivergences, hyphenation), [
      [7, range(7, "ariaLabel").column],
      [7, range(7, ":dataKey").column],
    ]);
    assert.ok(
      divergence.intentionalDivergences.every((entry: Finding) => entry.reason?.includes("aria")),
    );
    assert.deepEqual(pick(divergence.falseNegatives, hyphenation), []);
    assert.deepEqual(pick(divergence.falsePositives, hyphenation), []);
    assert.equal(
      divergence.shared.filter((entry: Finding) => entry.ruleId === hyphenation).length,
      1,
    );
    assert.deepEqual(pick(divergence.patinaOutsideBaselineSurface, "vue/no-unused-components"), [
      [2, range(2, "UnusedPanel").column],
    ]);
    assert.deepEqual(pick(divergence.falsePositives, "vue/no-unused-components"), []);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
