import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { normalizeKeyName, useKeyPressed, useMagicKeys } from "./magic-keys.ts";
import { keyEvent } from "./testing/fake-dom.ts";

void test("normalizes names and aliases", () => {
  assert.equal(normalizeKeyName("Control"), "control");
  assert.equal(normalizeKeyName("ctrl"), "control");
  assert.equal(normalizeKeyName(" "), "space");
  assert.equal(normalizeKeyName("Esc"), "escape");
});

void test("tracks combos from key and code names and releases on keyup", () => {
  const target = new EventTarget();
  const scope = effectScope();
  const keys = scope.run(() => useMagicKeys({ target }));
  assert.ok(keys);
  const save = keys.isPressed("ctrl+s");
  const { shift, ctrl_keys: ctrlByCode } = keys.combos;
  assert.ok(shift && ctrlByCode);

  target.dispatchEvent(keyEvent("keydown", "Control", "ControlLeft"));
  target.dispatchEvent(keyEvent("keydown", "s", "KeyS"));
  assert.equal(save.value, true);
  assert.equal(ctrlByCode.value, true);
  assert.equal(shift.value, false);
  assert.ok(keys.current.has("controlleft"));
  assert.equal(keys.isPressed("ctrl+s"), save);

  target.dispatchEvent(keyEvent("keyup", "s", "KeyS"));
  assert.equal(save.value, false);
  scope.stop();
  assert.equal(keys.current.size, 0);
});

void test("modifier release clears swallowed keys and blur resets everything", () => {
  const target = new EventTarget();
  const fired: string[] = [];
  const keys = useMagicKeys({
    target,
    aliasMap: { Save: "s" },
    onEventFired: (event) => fired.push(event.type),
  });
  const combo = keys.isPressed("meta+save");

  target.dispatchEvent(keyEvent("keydown", "Meta", "MetaLeft"));
  target.dispatchEvent(keyEvent("keydown", "s", "KeyS"));
  assert.equal(combo.value, true);
  target.dispatchEvent(keyEvent("keyup", "Meta", "MetaLeft"));
  assert.equal(keys.current.size, 0);

  target.dispatchEvent(keyEvent("keydown", "a", "KeyA"));
  target.dispatchEvent(new Event("blur"));
  assert.equal(keys.current.size, 0);
  assert.deepEqual(fired, ["keydown", "keydown", "keyup", "keydown"]);
  keys.reset();
});

void test("useKeyPressed matches names, lists, and predicates", () => {
  const target = new EventTarget();
  const scope = effectScope();
  const escape = scope.run(() => useKeyPressed("esc", { target }));
  const arrows = scope.run(() => useKeyPressed(["up", "down"], { target }));
  const digit = scope.run(() => useKeyPressed((event) => /^\d$/.test(event.key), { target }));

  target.dispatchEvent(keyEvent("keydown", "Escape", "Escape"));
  target.dispatchEvent(keyEvent("keydown", "ArrowUp", "ArrowUp"));
  target.dispatchEvent(keyEvent("keydown", "ArrowDown", "ArrowDown"));
  target.dispatchEvent(keyEvent("keydown", "7", "Digit7"));
  assert.deepEqual([escape?.value, arrows?.value, digit?.value], [true, true, true]);

  target.dispatchEvent(keyEvent("keyup", "ArrowUp", "ArrowUp"));
  assert.equal(arrows?.value, true);
  target.dispatchEvent(keyEvent("keyup", "ArrowDown", "ArrowDown"));
  assert.equal(arrows?.value, false);
  target.dispatchEvent(new Event("blur"));
  assert.deepEqual([escape?.value, digit?.value], [false, false]);
  scope.stop();
});

void test("nothing is pressed without a target", () => {
  const keys = useMagicKeys({ target: null });
  assert.equal(keys.isPressed("a").value, false);
  assert.equal(useKeyPressed("a", { target: null }).value, false);
});
