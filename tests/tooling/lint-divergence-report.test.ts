/**
 * The corpus runner behind `tools/fixtures/lint-divergence.mjs`.
 *
 * `tests/tooling/lint-divergence*.test.ts` cover the comparator's classification.
 * This suite covers what stands between the comparator and a real repository —
 * the parts that decide whether a reported number means anything: which rules
 * are comparable under the preset actually run, that both linters read the same
 * corpus, and that a corpus source's `eslint-disable` comment naming a foreign
 * toolchain's rule is not mistaken for rule-map drift.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  baselineConfig,
  resolveBaselineRuntime,
  retainEnabledFindings,
  selectComparableRules,
} from "../../tools/fixtures/lint-divergence-baseline.mjs";
import { renderMarkdown } from "../../tools/fixtures/lint-divergence-markdown.mjs";
import {
  reconcileCorpus,
  runLintDivergenceReport,
} from "../../tools/fixtures/lint-divergence-report.mjs";
import { readRuleMap } from "../../tools/fixtures/patina-rule-map.mjs";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const vizeBin = process.env.VIZE_TEST_BIN ?? process.env.VIZE_BIN ?? null;

const ruleMap = {
  upstream: { package: "eslint-plugin-vue", version: "10.9.2", ruleCount: 4 },
  entries: {
    "vue/no-v-html": {
      status: "mapped",
      patinaRule: "vue/no-v-html",
      patinaSeverity: "warning",
      patinaPresets: ["ecosystem", "opinionated"],
    },
    "vue/require-v-for-key": {
      status: "mapped",
      patinaRule: "vue/require-v-for-key",
      patinaSeverity: "error",
      patinaPresets: ["ecosystem"],
    },
    "vue/no-multiple-template-root": {
      status: "mapped",
      patinaRule: "vue/no-multiple-template-root",
      patinaSeverity: "error",
      patinaPresets: [],
    },
    "vue/no-undef-properties": { status: "unimplemented", issue: 3223 },
  },
};

test("only mapped rules the preset activates are comparable, at patina's severity", () => {
  const { rules, skippedByPreset } = selectComparableRules(ruleMap, "ecosystem");

  assert.deepEqual(rules, {
    "vue/no-v-html": "warn",
    "vue/require-v-for-key": "error",
  });
  // A rule no preset enables cannot produce a patina finding, so leaving it in
  // the baseline would report every upstream finding for it as a false negative.
  assert.deepEqual(skippedByPreset, [
    { upstreamRule: "vue/no-multiple-template-root", patinaRule: "vue/no-multiple-template-root" },
  ]);
});

test("preset filtering can be waived, and the coverage gap can be measured", () => {
  assert.deepEqual(Object.keys(selectComparableRules(ruleMap, null).rules).sort(), [
    "vue/no-multiple-template-root",
    "vue/no-v-html",
    "vue/require-v-for-key",
  ]);

  const withGap = selectComparableRules(ruleMap, "ecosystem", { includeUnimplemented: true });
  assert.equal(withGap.rules["vue/no-undef-properties"], "warn");
  assert.equal(withGap.rules["vue/require-v-for-key"], "error");
});

test("the baseline keeps inline directives and drops foreign-rule complaints", () => {
  const [config] = baselineConfig(
    { plugin: {}, vueParser: {}, scriptParser: {} },
    { "vue/no-v-html": "warn" },
  );
  assert.deepEqual(config.linterOptions, { reportUnusedDisableDirectives: "off" });
  assert.notEqual(config.linterOptions.noInlineConfig, true);

  const { results, droppedConfigMessageCount } = retainEnabledFindings(
    [
      {
        filePath: "/w/App.vue",
        messages: [
          { ruleId: "vue/no-v-html", severity: 1, line: 1, column: 1 },
          { ruleId: "@typescript-eslint/no-unused-vars", severity: 2, line: 1, column: 1 },
          { ruleId: null, severity: 2, line: 1, column: 1, message: "Parsing error" },
        ],
      },
    ],
    { "vue/no-v-html": "warn" },
  );

  assert.equal(droppedConfigMessageCount, 1);
  assert.deepEqual(
    results[0].messages.map((message: { ruleId: string | null }) => message.ruleId),
    ["vue/no-v-html", null],
    "a parse failure must survive: the comparator counts it as excluded evidence",
  );
});

test("the checked-in rule map carries everything the baseline needs", () => {
  const mapped = Object.values(readRuleMap().entries).filter(
    (entry: { status: string }) => entry.status === "mapped",
  ) as Array<{ patinaSeverity: string; patinaPresets: string[] }>;

  assert.ok(mapped.length > 0);
  for (const entry of mapped) {
    assert.match(entry.patinaSeverity, /^(?:error|warning)$/u);
    assert.ok(Array.isArray(entry.patinaPresets));
  }
  assert.ok(
    Object.keys(selectComparableRules(readRuleMap(), "ecosystem").rules).length > 0,
    "the ecosystem preset must leave a non-empty comparable surface",
  );
});

test("a run that compared no rule says so in the summary a reviewer reads", () => {
  const markdown = renderMarkdown({
    project: "fixture",
    revision: "0".repeat(40),
    preset: "incremental",
    evidence: { commitSha: "1".repeat(40) },
    files: { comparedCount: 3 },
    baseline: {
      package: "eslint-plugin-vue",
      version: "10.9.2",
      comparedRuleCount: 0,
      mappedRuleCount: 123,
      droppedConfigMessageCount: 0,
    },
    divergence: {
      summary: emptySummary(),
      falsePositives: [],
      falseNegatives: [],
      unimplemented: [],
    },
  });

  assert.match(markdown, /No mapped rule was comparable under this preset/u);
  assert.match(markdown, /### False positives: none/u);
});

test("the markdown breaks findings down by upstream rule", () => {
  const markdown = renderMarkdown({
    project: "fixture",
    revision: "0".repeat(40),
    preset: "ecosystem",
    evidence: { commitSha: "1".repeat(40) },
    files: { comparedCount: 1 },
    baseline: {
      package: "eslint-plugin-vue",
      version: "10.9.2",
      comparedRuleCount: 2,
      mappedRuleCount: 123,
      droppedConfigMessageCount: 4,
    },
    divergence: {
      summary: { ...emptySummary(), falsePositiveCount: 2, ruleLocationDivergenceCount: 1 },
      falsePositives: [
        { ruleId: "vue/no-v-html", upstreamRuleId: "vue/no-v-html" },
        { ruleId: "vue/no-v-html", upstreamRuleId: "vue/no-v-html" },
      ],
      falseNegatives: [{ ruleId: "vue/attribute-order", upstreamRuleId: "vue/attributes-order" }],
      ruleLocationDivergences: [
        { ruleId: "vue/require-v-for-key", upstreamRuleId: "vue/require-v-for-key" },
      ],
      unimplemented: [],
    },
  });

  assert.match(markdown, /\| `vue\/no-v-html` \| 2 \|/u);
  assert.match(markdown, /\| `vue\/attributes-order` \| 1 \|/u);
  assert.match(markdown, /\| `vue\/require-v-for-key` \| 1 \|/u);
  assert.match(markdown, /Rule location divergences: 1/u);
  assert.match(markdown, /dropped as foreign-rule directives: 4/u);
});

test("the runner classifies a real divergence over a synthetic pinned project", async (t) => {
  if (vizeBin == null) {
    t.skip("set VIZE_TEST_BIN to exercise the runner against the real binary");
    return;
  }
  const runtime = resolveBaselineRuntime();
  assert.equal(runtime.version, readRuleMap().upstream.version);

  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-divergence-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-divergence-report-"));
  try {
    // `v-html`, which both linters flag, plus a foreign-toolchain `eslint-disable`
    // comment of the kind every real corpus source carries: ESLint answers
    // "Definition for rule ... was not found" with that rule as the `ruleId`.
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      [
        "<script setup>",
        "// eslint-disable-next-line @typescript-eslint/no-explicit-any",
        "const content = 1",
        "</script>",
        "<template>",
        '  <article v-html="content" />',
        "</template>",
        "",
      ].join("\n"),
    );
    const registryPath = path.join(fixtureDir, "registry.json");
    fs.writeFileSync(
      registryPath,
      JSON.stringify({
        projects: [
          {
            id: "lint-divergence-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["**/*.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const [artifact] = await runLintDivergenceReport([
      "--registry",
      registryPath,
      "--output-dir",
      outputDir,
      "--vize-bin",
      vizeBin,
    ]);

    assert.equal(artifact.schema, "vize.fixtureLintDivergenceRun");
    assert.equal(artifact.version, 2);
    assert.equal(artifact.project, "lint-divergence-fixture");
    assert.equal(artifact.files.comparedCount, 1);
    assert.equal(
      artifact.baseline.droppedConfigMessageCount,
      1,
      "the foreign-toolchain directive must be dropped, not read as rule-map drift",
    );
    // Both linters flag the same `v-html`, so it lands in `shared`, not in a
    // divergence bucket. That is the assertion worth pinning: the runner proves
    // agreement, not merely that it produced a report.
    assert.equal(artifact.divergence.summary.sharedCount, 1);
    assert.equal(artifact.divergence.summary.falsePositiveCount, 0);
    assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveCount: 0,
      maxFalseNegativeCount: 0,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "passed",
      passed: true,
    });
    assert.deepEqual(
      artifact.divergence.shared.map((pair: { ruleId: string }) => pair.ruleId),
      ["vue/no-v-html"],
    );

    const index = JSON.parse(
      fs.readFileSync(path.join(outputDir, "lint-divergence-summary.json"), "utf8"),
    );
    assert.equal(index.schema, "vize.fixtureLintDivergenceIndex");
    assert.equal(index.projectCount, 1);
    assert.deepEqual(index.budget, {
      status: "success",
      passed: true,
      projectCount: 1,
      passedCount: 1,
      failedCount: 0,
      unusableCount: 0,
      breachedCount: 0,
      failedProjects: [],
    });
    assert.equal(index.totals.sharedCount, 1);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("a corpus the two linters did not both read is refused, not compared", () => {
  const cwd = path.join(repoRoot, "target", "reconcile-fixture");
  const read = (file: string) => ({ filePath: path.join(cwd, file), messages: [] });

  assert.doesNotThrow(() =>
    reconcileCorpus(
      "fixture",
      ["App.vue", "src/Card.vue"],
      [read("App.vue"), read("src/Card.vue")],
      cwd,
    ),
  );
  // Findings in a file the baseline never opened would all read as false
  // negatives, so the skew has to surface as a failure instead.
  assert.throws(
    () => reconcileCorpus("fixture", ["App.vue", "src/Card.vue"], [read("App.vue")], cwd),
    /fixture: baseline skipped 1 of 2 files, starting with src\/Card\.vue/u,
  );
  // Extra baseline files are not a skew: the comparator classifies by identity,
  // and a file patina did not report simply has no candidate findings.
  assert.doesNotThrow(() =>
    reconcileCorpus("fixture", ["App.vue"], [read("App.vue"), read("extra.vue")], cwd),
  );
});

function emptySummary() {
  return {
    patinaFindingCount: 0,
    baselineFindingCount: 0,
    comparableBaselineCount: 0,
    sharedCount: 0,
    messageDifferenceCount: 0,
    documentedDivergenceCount: 0,
    ruleLocationDivergenceCount: 0,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
    unimplementedCount: 0,
    intentionalDivergenceCount: 0,
    patinaOnlyRuleFindingCount: 0,
    baselineParseErrorCount: 0,
    baselineInvalidRangeCount: 0,
    falsePositiveRatio: 0,
    falseNegativeRatio: 0,
  };
}
