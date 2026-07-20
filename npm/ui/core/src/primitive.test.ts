import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const source = await readFile(new URL("./PrimitiveElement.vue", import.meta.url), "utf8");

void test("ships a polymorphic SFC without render-function escape hatches", () => {
  assert.match(source, /<component :is="as"/);
  assert.match(source, /readonly as\?: PrimitiveAs/);
  assert.match(source, /@default "div"/);
  assert.doesNotMatch(source, /\bh\s*\(/);
  assert.doesNotMatch(source, /defineOptions|withDefaults|interface (?:Props|Emits)/);
});

void test("forwards every slot and exposes the rendered value deliberately", () => {
  assert.match(source, /v-for="name in getSlotNames\(\)"/);
  assert.match(source, /<slot :name="name" \/>/);
  assert.match(source, /defineExpose\(\{ element \}\)/);
  assert.match(source, /data-vize-ui="primitive"/);
});
