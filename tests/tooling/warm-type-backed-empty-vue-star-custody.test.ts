import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { runRetainedEmptyVueStarCustody } from "../../tools/support/compat/github/empty-vue-star-custody-runner.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const result = await runRetainedEmptyVueStarCustody(root);
void test("actual JS93 and Star322 source gates precede all eight unchanged original Star custody laws", () => {
  assert.equal(result.registeredOriginalTests, 8);
  assert.equal(result.currentBodies, 322);
  assert.equal(result.sourceOnly, true);
  assert.equal(result.nativeCredit, false);
  assert.equal(result.original400Credit, false);
  assert.equal(result.performanceCredit, false);
});
