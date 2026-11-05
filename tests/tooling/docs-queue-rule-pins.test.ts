import assert from "node:assert/strict";
import { test } from "node:test";

import { ruleRouteKind, verifyRuleSources } from "../../docs/scripts/rule-render-assertions.ts";

void test("the queue's docs review also runs the release review's source-side rule pins", () => {
  assert.equal(ruleRouteKind("/rules/vue"), "index");
  assert.equal(ruleRouteKind("/fr/rules/cross-file"), "index");
  assert.equal(ruleRouteKind("/ja/rules/all"), "all");
  assert.equal(ruleRouteKind("/fr/rules/all"), "all");
  assert.equal(ruleRouteKind("/rules/reference/vue-valid-v-on"), null);
  assert.equal(ruleRouteKind("/guide/cli"), null);
  for (const route of [
    "/rules/vue",
    "/ja/rules/vue",
    "/rules/all",
    "/ja/rules/all",
    "/fr/rules/all",
  ]) {
    assert.doesNotThrow(() => verifyRuleSources(route), route);
  }
});
