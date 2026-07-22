import assert from "node:assert/strict";
import { test } from "node:test";

import { useToggle } from "./use-toggle.ts";

void test("starts false and inverts on every call", () => {
  const { state, toggle } = useToggle();

  assert.equal(state.value, false);
  assert.equal(toggle(), true);
  assert.equal(state.value, true);
  assert.equal(toggle(), false);
  assert.equal(state.value, false);
});

void test("honors the initial value", () => {
  const { state, toggle } = useToggle(true);

  assert.equal(state.value, true);
  assert.equal(toggle(), false);
});

void test("forces the state when an argument is given", () => {
  const { state, toggle } = useToggle();

  assert.equal(toggle(true), true);
  assert.equal(toggle(true), true);
  assert.equal(state.value, true);
  assert.equal(toggle(false), false);
  assert.equal(state.value, false);
});

void test("treats an explicit undefined argument as a plain inversion", () => {
  const { state, toggle } = useToggle();

  assert.equal(toggle(undefined), true);
  assert.equal(toggle(undefined), false);
  assert.equal(state.value, false);
});

void test("owns writable state that direct assignment and toggling share", () => {
  const { state, toggle } = useToggle();

  state.value = true;
  assert.equal(toggle(), false);
  state.value = true;
  assert.equal(state.value, true);
});
