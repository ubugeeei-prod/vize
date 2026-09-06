import assert from "node:assert/strict";
import { test } from "node:test";
import { readRuleMap, validateRuleMap } from "../../legacy-tools/fixtures/patina-rule-map.mjs";

test("the Patina scorecard exhaustively maps the pinned eslint-plugin-vue rule surface", () => {
  const ruleMap = validateRuleMap(readRuleMap());

  assert.equal(ruleMap.upstream.version, "10.9.2");
  assert.equal(ruleMap.upstream.ruleCount, 252);
  assert.deepEqual(ruleMap.summary, { mapped: 123, unimplemented: 127, intentionalDivergence: 2 });
  assert.deepEqual(ruleMap.entries["vue/component-definition-name-casing"], {
    status: "intentional-divergence",
    reason:
      "Patina's rule checks SFC file-name casing; eslint-plugin-vue checks the component definition name, so their findings are not comparable.",
  });
  assert.deepEqual(ruleMap.entries["vue/no-unused-properties"], {
    status: "intentional-divergence",
    reason:
      "Patina intentionally checks defineProps declarations only; eslint-plugin-vue also checks Options API props, so the surfaces are not comparable.",
  });
});
