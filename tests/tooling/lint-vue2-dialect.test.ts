import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  vue2DialectDisabledRules,
  vue2DialectReason,
} from "../../tools/support/fixtures/lint_divergence/vue2_dialect.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("the Vue 2 dialect rule list is read from Patina", () => {
  const rules = vue2DialectDisabledRules(root);
  for (const rule of [
    "vue/no-deprecated-filter",
    "vue/no-deprecated-slot-attribute",
    "script/no-deprecated-destroyed-lifecycle",
    "script/require-slots-as-functions",
  ])
    assert.ok(rules.has(rule), rule);
  for (const rule of ["vue/require-v-for-key", "vue/no-unused-vars", "vue/next-tick-style"])
    assert.ok(!rules.has(rule), rule);
  const source = fs.readFileSync(
    path.join(root, "crates/vize_patina/src/linter/compatibility.rs"),
    "utf8",
  );
  const inserted = [...source.matchAll(/String::from\("([^"]+)"\)/gu)].map((match) => match[1]);
  // Only the legacy block is read: the Vapor block's rules stay out.
  assert.ok(inserted.includes("script/no-next-tick"));
  assert.ok(!rules.has("script/no-next-tick"));
  assert.match(vue2DialectReason, /Vue 2 dialect/u);
});
