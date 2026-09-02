import assert from "node:assert/strict";
import { test } from "node:test";

import { createFormatterChangeEvidence } from "../../legacy-tools/fixtures/tool-matrix-formatter.mjs";

test("formatter changed-path evidence is byte-ordered for Rust parity", () => {
  assert.equal(
    createFormatterChangeEvidence(3, ["src/ä.vue", "src/z.vue", "src/a.vue"]).changedPathsSha256,
    "91ac181cb4e41902d4ba6c932b5a661be5cc0e3e535f633c6426e81d9d10685c",
  );
});
