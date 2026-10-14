import assert from "node:assert/strict";
import { test } from "node:test";

import { ruleRouteKind, verifyRuleSources } from "../../docs/scripts/rule-render-assertions.ts";

void test("the queue's docs review also runs the release review's source-side rule pins", () => {
  assert.equal(ruleRouteKind("/rules/vue"), "vue");
  assert.equal(ruleRouteKind("/ja/rules/all"), "references");
  assert.equal(ruleRouteKind("/fr/rules/all"), "overview");
  assert.equal(ruleRouteKind("/guide/cli"), null);
  for (const route of ["/rules/vue", "/ja/rules/vue", "/rules/all", "/ja/rules/all"]) {
    assert.doesNotThrow(() => verifyRuleSources(route), route);
  }
});
