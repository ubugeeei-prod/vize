import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import { getButtonKeyboardAction } from "./button-keyboard.ts";

const source = await readFile(new URL("./ActionButton.vue", import.meta.url), "utf8");

void test("matches native keyboard activation timing", () => {
  assert.equal(getButtonKeyboardAction("Enter", "keydown"), "activate");
  assert.equal(getButtonKeyboardAction("Enter", "keyup"), "ignore");
  assert.equal(getButtonKeyboardAction(" ", "keydown"), "prevent");
  assert.equal(getButtonKeyboardAction(" ", "keyup"), "activate");
  assert.equal(getButtonKeyboardAction("Escape", "keydown"), "ignore");
});

void test("ships typed disabled and loading semantics in an explicit SFC", () => {
  assert.match(source, /defineEmits<\{[\s\S]*press: \[event: MouseEvent\]/);
  assert.match(source, /:disabled="isNativeButton \? disabled : undefined"/);
  assert.match(source, /:aria-disabled=/);
  assert.match(source, /:aria-busy=/);
  assert.match(source, /:tabindex="tabIndex"/);
  assert.doesNotMatch(source, /\bh\s*\(/);
  assert.doesNotMatch(source, /defineOptions|withDefaults|interface (?:Props|Emits)/);
});

void test("exposes focus deliberately and provides programmable slot state", () => {
  assert.match(source, /defineExpose\(\{ element, focus \}\)/);
  assert.match(source, /<slot :disabled :loading :unavailable \/>/);
  assert.match(source, /data-vize-ui="button"/);
  assert.match(source, /:data-state=/);
});
