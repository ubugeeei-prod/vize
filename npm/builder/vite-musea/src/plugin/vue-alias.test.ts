import assert from "node:assert/strict";
import test from "node:test";

import { createVueRuntimeCompilerAlias } from "./vue-alias.js";

void test("Vue runtime compiler alias only matches the bare vue import", () => {
  const alias = createVueRuntimeCompilerAlias();
  assert.equal(alias.find.test("vue"), true);
  assert.equal(alias.find.test("vue/server-renderer"), false);
  assert.equal(alias.find.test("vue/dist/vue.runtime.esm-bundler.js"), false);
  assert.match(alias.replacement, /vue[\\/]dist[\\/]vue\.esm-bundler\.js$/);
});
