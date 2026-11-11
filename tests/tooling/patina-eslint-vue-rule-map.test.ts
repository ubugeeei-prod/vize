import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readRuleMap,
  validateRuleMap,
} from "../../tools/support/compat/fixtures/patina-rule-map.ts";

test("the Patina scorecard exhaustively maps the pinned eslint-plugin-vue rule surface", () => {
  const ruleMap = validateRuleMap(readRuleMap());

  assert.equal(ruleMap.upstream.version, "10.12.0");
  assert.equal(ruleMap.upstream.ruleCount, 256);
  assert.deepEqual(ruleMap.summary, { mapped: 185, unimplemented: 68, intentionalDivergence: 3 });
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
  assert.deepEqual(ruleMap.entries["vue/v-on-handler-style"], {
    status: "intentional-divergence",
    reason:
      "Patina's opinionated policy permits only method references, while eslint-plugin-vue always permits two of method, inline, and inline-function handlers; no upstream option has the same acceptance surface.",
  });
});
