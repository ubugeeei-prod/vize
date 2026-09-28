import assert from "node:assert/strict";
import { test } from "node:test";

import {
  buildDialectInputs,
  checkDialectInputs,
} from "../../tools/support/compat/fixtures/generate-dialect-inputs.mjs";

void test("rare dialect input inventory is generated from pinned grammar rules", () => {
  checkDialectInputs();
  const { inventory } = buildDialectInputs();
  assert.equal(inventory.cases.length, 7);
  assert.deepEqual(
    [...new Set(inventory.cases.flatMap((item) => item.dialects))].sort((a, b) =>
      a < b ? -1 : a > b ? 1 : 0,
    ),
    ["vue-quirks", "vue0-template", "vue1-template"],
  );
  assert.ok(inventory.cases.every((item) => item.state === "input-only"));
  assert.equal(
    inventory.cases[0].input.sha256,
    inventory.cases[1].input.sha256,
    "Vue 0.10 and 0.11 share syntax but retain distinct version selectors",
  );
});
