import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { runRetainedComponentReadStarLaws } from "../../tools/support/compat/github/empty-vue-star-component-read-source.mjs";
const result = await runRetainedComponentReadStarLaws(path.resolve(import.meta.dirname, "../.."));
void test("complete current component-read counterpart precedes all nine unchanged original Star source laws", () => {
  assert.equal(result.closureBodies, 415);
  assert.equal(result.sourceOnly, true);
  assert.equal(result.nativeCredit, false);
  assert.equal(result.original400Credit, false);
  assert.equal(result.performanceCredit, false);
});
