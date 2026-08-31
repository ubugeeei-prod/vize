import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { compareLintFindings } from "../../tools/fixtures/lint-divergence.mjs";
import { readRuleMap } from "../../tools/fixtures/patina-rule-map.mjs";
import {
  compare,
  cwd,
  eslintResult,
  ledgerPath,
  span,
} from "./_helpers/lint-divergence-fixture.ts";

test("a ledger entry cancels exactly one false positive against one false negative", () => {
  const patina = [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      severity: 2,
      ...span(4, 6, 4, 18),
      message: "v-html",
    },
  ];
  const baseline = [
    eslintResult("src/App.vue", [
      {
        ruleId: "vue/no-v-html",
        severity: 2,
        ...span(4, 6, 4, 30),
        message: "`v-html` directive can lead to XSS attack.",
      },
    ]),
  ];
  const reason =
    "patina highlights the directive itself while eslint-plugin-vue highlights the whole attribute.";
  const ledger = [
    {
      project: "fixture",
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      patina: { severity: "error", ...span(4, 6, 4, 18) },
      baseline: { severity: "error", ...span(4, 6, 4, 30) },
      issue: 3223,
      reason,
    },
  ];

  const undocumented = compare(patina, baseline);
  assert.equal(undocumented.summary.falsePositiveCount, 1);
  assert.equal(undocumented.summary.falseNegativeCount, 1);

  const documented = compare(patina, baseline, ledger);
  assert.deepEqual(documented.falsePositives, []);
  assert.deepEqual(documented.falseNegatives, []);
  assert.deepEqual(documented.documentedDivergences, [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      patina: { severity: "error", line: 4, column: 6, endLine: 4, endColumn: 18 },
      baseline: { severity: "error", line: 4, column: 6, endLine: 4, endColumn: 30 },
      issue: 3223,
      reason,
    },
  ]);
  assert.equal(documented.summary.documentedDivergenceCount, 1);
});

test("a ledger entry only cancels a divergence that is actually present", () => {
  // The ledger describes a divergence in a file neither linter reported on, so
  // it must stay unpaired rather than silently absorbing something else.
  const documented = compare(
    [],
    [],
    [
      {
        project: "fixture",
        file: "src/Other.vue",
        ruleId: "vue/no-v-html",
        patina: { severity: "error", ...span(1, 1, 1, 4) },
        baseline: { severity: "error", ...span(2, 1, 2, 4) },
        issue: 3223,
        reason:
          "patina reports the directive name while eslint-plugin-vue reports the attribute value node.",
      },
    ],
  );

  assert.deepEqual(documented.documentedDivergences, []);
  assert.equal(documented.summary.documentedDivergenceCount, 0);
});

test("a ledger entry that records no actual difference is rejected", () => {
  assert.throws(
    () =>
      compare(
        [],
        [],
        [
          {
            project: "fixture",
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            patina: { severity: "error", ...span(1, 1, 1, 2) },
            baseline: { severity: "error", ...span(1, 1, 1, 2) },
            issue: 3223,
            reason:
              "this rationale is long enough to pass the length gate but describes no divergence.",
          },
        ],
      ),
    /must record a difference between the two linters/,
  );
});

test("a ledger entry without a written rationale is rejected", () => {
  assert.throws(
    () =>
      compare(
        [],
        [],
        [
          {
            project: "fixture",
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            patina: { severity: "error", ...span(1, 1, 1, 2) },
            baseline: { severity: "error", ...span(2, 1, 2, 2) },
            issue: 3223,
            reason: "different",
          },
        ],
      ),
    /reason must explain why the divergence is expected/,
  );
});

test("a ledger entry without a tracking issue is rejected", () => {
  assert.throws(
    () =>
      compare(
        [],
        [],
        [
          {
            project: "fixture",
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            patina: { severity: "error", ...span(1, 1, 1, 2) },
            baseline: { severity: "error", ...span(2, 1, 2, 2) },
            reason:
              "patina reports the directive name while eslint-plugin-vue reports the attribute value node.",
          },
        ],
      ),
    /issue must be the tracking issue number/,
  );
});

test("the checked-in divergence ledger validates against the checked-in rule map", () => {
  const ledger = JSON.parse(fs.readFileSync(ledgerPath, "utf8"));
  assert.ok(Array.isArray(ledger), "the ledger must be an array of divergence entries");
  const result = compareLintFindings({
    projectId: "unhydrated",
    cwd,
    ruleMap: readRuleMap(),
    patinaFindings: [],
    eslintResults: [],
    documentedDivergences: ledger,
  });

  assert.equal(result.schema, "vize.fixtureLintDivergence");
  assert.equal(result.version, 1);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);
});
