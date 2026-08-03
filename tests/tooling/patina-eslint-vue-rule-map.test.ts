import assert from "node:assert/strict";
import { test } from "node:test";
import { readRuleMap, validateRuleMap } from "../../tools/fixtures/patina-rule-map.mjs";

test("the Patina scorecard exhaustively maps the pinned eslint-plugin-vue rule surface", () => {
  const ruleMap = validateRuleMap(readRuleMap());

  assert.equal(ruleMap.upstream.version, "10.9.2");
  assert.equal(ruleMap.upstream.ruleCount, 252);
  assert.deepEqual(ruleMap.summary, { mapped: 123, unimplemented: 129 });
});
