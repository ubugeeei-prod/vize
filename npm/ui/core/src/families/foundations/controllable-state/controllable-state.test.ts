import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import { useControllableState } from "./controllable-state.ts";

test("updates and resets uncontrolled state", () => {
  const changes: [number, number][] = [];
  const state = useControllableState({
    defaultValue: 1,
    onChange: (value, previous) => changes.push([value, previous]),
  });

  assert.equal(state.controlled.value, false);
  assert.equal(
    state.set((value) => value + 1),
    true,
  );
  assert.equal(state.value.value, 2);
  assert.equal(state.reset(), true);
  assert.deepEqual(changes, [
    [2, 1],
    [1, 2],
  ]);
});

test("requests controlled updates without mutating the source", () => {
  const source = ref<boolean | undefined>(false);
  let requested: boolean | undefined;
  const state = useControllableState({
    value: source,
    defaultValue: true,
    onChange: (value) => (requested = value),
  });

  assert.equal(state.controlled.value, true);
  assert.equal(state.set(true), true);
  assert.equal(source.value, false);
  assert.equal(state.value.value, false);
  assert.equal(requested, true);
  source.value = true;
  assert.equal(state.value.value, true);
});

test("retains the last value when control is released", () => {
  const scope = effectScope();
  const source = ref<string | undefined>("controlled");
  const state = scope.run(() => useControllableState({ value: source, defaultValue: "initial" }));
  assert.ok(state);

  source.value = "latest";
  source.value = undefined;
  assert.equal(state.controlled.value, false);
  assert.equal(state.value.value, "latest");
  scope.stop();
});

test("supports domain-specific equality and reports redundant updates", () => {
  const state = useControllableState({
    defaultValue: { id: 1, label: "first" },
    equals: (left, right) => left.id === right.id,
  });

  assert.equal(state.set({ id: 1, label: "equivalent" }), false);
  assert.equal(state.value.value.label, "first");
});
