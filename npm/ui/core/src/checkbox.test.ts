import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import { getCheckboxState } from "./checkbox-state.ts";

const source = await readFile(new URL("./CheckboxControl.vue", import.meta.url), "utf8");

void test("gives the mixed visual state precedence", () => {
  assert.equal(getCheckboxState(false, false), "unchecked");
  assert.equal(getCheckboxState(true, false), "checked");
  assert.equal(getCheckboxState(false, true), "indeterminate");
  assert.equal(getCheckboxState(true, true), "indeterminate");
});

void test("ships native form and mixed-state semantics in an explicit SFC", () => {
  assert.match(source, /type="checkbox"/);
  assert.match(source, /:aria-checked="indeterminate \? 'mixed' : checked"/);
  assert.match(source, /element\.value\.indeterminate = indeterminate/);
  assert.match(source, /form\.addEventListener\("reset", onReset\)/);
  assert.match(source, /update:indeterminate/);
  assert.doesNotMatch(source, /\bh\s*\(/);
  assert.doesNotMatch(source, /defineOptions|withDefaults|interface (?:Props|Emits)/);
});

void test("exposes typed state and imperative controls deliberately", () => {
  assert.match(source, /defineExpose\(\{[\s\S]*setChecked: state\.set/);
  assert.match(source, /data-vize-ui="checkbox"/);
  assert.match(source, /:data-state="visualState"/);
});
