import assert from "node:assert/strict";
import { test } from "node:test";
import { readRuleMap, validateRuleMap } from "../../tools/fixtures/patina-rule-map.mjs";

test("the Patina scorecard exhaustively maps the pinned eslint-plugin-vue rule surface", () => {
  const ruleMap = validateRuleMap(readRuleMap());

  assert.equal(ruleMap.upstream.version, "10.9.2");
  assert.equal(ruleMap.upstream.ruleCount, 252);
  assert.ok(ruleMap.summary.mapped > 100, "the scorecard must expose existing Patina coverage");
  assert.ok(
    ruleMap.summary.unimplemented > 0,
    "the scorecard must keep uncovered upstream rules explicit",
  );
});
