import assert from "node:assert/strict";
import { test } from "node:test";

import { compare, eslintResult, span } from "./_helpers/lint-divergence-fixture.ts";

test("known rule-owned location anchors are classified without hiding both ranges", () => {
  const result = compare(
    [
      {
        file: "src/List.vue",
        ruleId: "vue/require-v-for-key",
        severity: 2,
        ...span(10, 7, 10, 41),
        message:
          "[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives. Element: <li>",
      },
    ],
    [
      eslintResult("src/List.vue", [
        {
          ruleId: "vue/require-v-for-key",
          severity: 2,
          ...span(10, 4, 10, 76),
          message: "Elements in iteration expect to have 'v-bind:key' directives.",
        },
      ]),
    ],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
  assert.deepEqual(result.ruleLocationDivergences, [
    {
      file: "src/List.vue",
      ruleId: "vue/require-v-for-key",
      upstreamRuleId: "vue/require-v-for-key",
      severity: "error",
      subject: "missing-v-for-key",
      reason:
        "patina reports the v-for directive range while eslint-plugin-vue reports the owning element range.",
      patina: {
        line: 10,
        column: 7,
        endLine: 10,
        endColumn: 41,
        message:
          "[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives. Element: <li>",
      },
      baseline: {
        line: 10,
        column: 4,
        endLine: 10,
        endColumn: 76,
        message: "Elements in iteration expect to have 'v-bind:key' directives.",
      },
    },
  ]);
  assert.equal(result.summary.ruleLocationDivergenceCount, 1);
  assert.equal(result.summary.falsePositiveRatio, 0);
  assert.equal(result.summary.falseNegativeRatio, 0);
});

test("unused component location anchors are paired by component identity", () => {
  const result = compare(
    [
      {
        file: "src/Uploader.vue",
        ruleId: "vue/no-unused-components",
        severity: 2,
        ...span(1, 11, 2, 12),
        message:
          "[vize:vue/no-unused-components] Component 'UploaderFile' is registered but never used in template",
      },
      {
        file: "src/Uploader.vue",
        ruleId: "vue/no-unused-components",
        severity: 2,
        ...span(1, 11, 2, 12),
        message:
          "[vize:vue/no-unused-components] Component 'UploaderFiles' is registered but never used in template",
      },
    ],
    [
      eslintResult("src/Uploader.vue", [
        {
          ruleId: "vue/no-unused-components",
          severity: 2,
          ...span(140, 7, 140, 20),
          message: 'The "UploaderFiles" component has been registered but not used.',
        },
        {
          ruleId: "vue/no-unused-components",
          severity: 2,
          ...span(141, 7, 141, 19),
          message: 'The "UploaderFile" component has been registered but not used.',
        },
      ]),
    ],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
  assert.deepEqual(
    result.ruleLocationDivergences.map((entry) => entry.subject),
    ["component:UploaderFile", "component:UploaderFiles"],
  );
  assert.equal(result.summary.ruleLocationDivergenceCount, 2);
});
