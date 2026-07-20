import assert from "node:assert/strict";
import { test } from "node:test";

import { formatSfcLintResults, lintSfcFiles } from "../scripts/lint-sfc.ts";

void test("keeps every shipped SFC clean under the opinionated Vize preset", async () => {
  const results = await lintSfcFiles();

  assert.deepEqual(
    results.map((result) => result.filename),
    ["src/VisuallyHidden.vue"],
  );
  assert.equal(formatSfcLintResults(results), "");
});
