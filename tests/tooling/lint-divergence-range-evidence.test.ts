import assert from "node:assert/strict";
import { test } from "node:test";

import { compare, eslintResult, span } from "./_helpers/lint-divergence-fixture.ts";

test("invalid baseline ranges retain the original payload beside valid findings", () => {
  const invalid = {
    ruleId: "vue/no-v-html",
    severity: 2,
    ...span(7, 1, 0, 0),
    message: "bad location",
    nodeType: "VAttribute",
    suggestions: [{ desc: "inspect the original node", data: { name: "html" } }],
  };
  const original = structuredClone(invalid);
  const result = compare(
    [],
    [
      eslintResult("src/App.vue", [
        invalid,
        {
          ruleId: "vue/require-v-for-key",
          severity: 2,
          ...span(9, 1, 9, 12),
          message: "missing key",
        },
      ]),
    ],
  );
  invalid.suggestions[0].data.name = "mutated later";
  assert.deepEqual(result.baselineInvalidRanges, [{ file: "src/App.vue", finding: original }]);
  assert.equal(result.summary.baselineInvalidRangeCount, 1);
  assert.equal(result.summary.baselineFindingCount, 1);
  assert.equal(result.falseNegatives.length, 1);
  assert.equal(result.falseNegatives[0].ruleId, "vue/require-v-for-key");
});

test("the evidence hash distinguishes original invalid coordinates with identical counts", () => {
  const resultAt = (column: number) =>
    compare(
      [],
      [
        eslintResult("src/App.vue", [
          {
            ruleId: "vue/no-v-html",
            severity: 2,
            ...span(7, column, 0, 0),
            message: "bad location",
          },
        ]),
      ],
    );
  const first = resultAt(1);
  const second = resultAt(5);
  assert.deepEqual(first.summary, second.summary);
  assert.notEqual(first.sha256, second.sha256);
  assert.equal(first.sha256, resultAt(1).sha256);
});

test("valid baseline evidence omits invalid payloads and preserves its original hash", () => {
  const result = compare([], [eslintResult("src/App.vue", [])]);
  assert.equal(Object.hasOwn(result, "baselineInvalidRanges"), false);
  assert.equal(result.summary.baselineInvalidRangeCount, 0);
  assert.equal(result.sha256, "69385a771996e2bf953a254b74b8021b7bfdd7d9a5d252d5a355fb43d3b9f31f");
});
