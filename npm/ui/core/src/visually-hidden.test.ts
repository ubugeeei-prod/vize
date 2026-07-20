import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const source = await readFile(new URL("./VisuallyHidden.vue", import.meta.url), "utf8");

void test("ships the primitive as an explicit opinionated SFC", () => {
  assert.match(source, /<script setup lang="ts">[\s\S]*<\/script>/);
  assert.match(source, /<template>[\s\S]*<\/template>/);
  assert.match(source, /<style scoped>[\s\S]*<\/style>/);
  assert.doesNotMatch(source, /\bh\s*\(/);
  assert.doesNotMatch(source, /defineOptions|withDefaults|interface (?:Props|Emits)/);
});

void test("preserves accessibility while removing visual layout", () => {
  assert.match(source, /<slot \/>/);
  assert.match(source, /position: absolute/);
  assert.match(source, /clip-path: inset\(50%\)/);
  assert.match(source, /white-space: nowrap/);
  assert.doesNotMatch(source, /display:\s*none|visibility:\s*hidden/);
});
