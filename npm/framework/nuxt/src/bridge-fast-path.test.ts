import assert from "node:assert/strict";
import { test } from "node:test";

import { hasI18nBridgeInput } from "./bridge-fast-path.ts";

void test("detects setup-scope i18n helpers after whitespace", () => {
  assert.equal(hasI18nBridgeInput("const label = $t('nav.compare')"), true);
  assert.equal(hasI18nBridgeInput("const count = $n(total)"), true);
});

void test("ignores modules without i18n helper calls", () => {
  assert.equal(hasI18nBridgeInput("const label = 'nav.compare'"), false);
});
