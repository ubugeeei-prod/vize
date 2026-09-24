import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, reactive } from "vue";

import { useVModel, useVModels } from "./use-v-model.ts";

void test("reads the prop and emits updates", () => {
  const props = reactive({ modelValue: "a" });
  const emitted: [string, string][] = [];
  const model = useVModel(props, "modelValue", (event: "update:modelValue", value: string) => {
    emitted.push([event, value]);
  });

  assert.equal(model.value, "a");
  model.value = "b";
  assert.deepEqual(emitted, [["update:modelValue", "b"]]);
  assert.equal(model.value, "a");
  props.modelValue = "c";
  assert.equal(model.value, "c");
});

void test("custom events, defaults, and emit guards", () => {
  const props = reactive<{ count?: number }>({});
  const emitted: number[] = [];
  const model = useVModel(
    props,
    "count",
    (_event: "change", value: number | undefined) => {
      emitted.push(value ?? -1);
    },
    { eventName: "change", defaultValue: 5, shouldEmit: (value) => (value ?? 0) >= 0 },
  );

  assert.equal(model.value, 5);
  model.value = -1;
  model.value = 2;
  assert.deepEqual(emitted, [2]);
});

void test("passive mode keeps local state and follows the prop", async () => {
  const props = reactive({ value: { n: 1 } });
  const emitted: { n: number }[] = [];
  const model = useVModel(
    props,
    "value",
    (_event: "update:value", value: { n: number }) => {
      emitted.push(value);
    },
    { passive: true, clone: true },
  );

  assert.notEqual(model.value, props.value);
  model.value = { n: 2 };
  assert.deepEqual(emitted, [{ n: 2 }]);
  assert.equal(model.value.n, 2);
  props.value = { n: 3 };
  await nextTick();
  assert.equal(model.value.n, 3);
  assert.equal(emitted.length, 1);
});

void test("passive deep mode emits on nested mutation", async () => {
  const props = reactive({ list: [1] });
  let emits = 0;
  const model = useVModel(
    props,
    "list",
    () => {
      emits += 1;
    },
    { passive: true, deep: true, clone: (value) => [...value] },
  );
  model.value.push(2);
  assert.equal(emits, 1);
  assert.deepEqual(props.list, [1], "the parent's array is never mutated");
  model.value = [...model.value, 3];
  assert.equal(emits, 2);
});

void test("useVModels binds several props", () => {
  const props = reactive({ open: false, title: "x", readonlyProp: 1 });
  const emitted: [string, unknown][] = [];
  function emit(event: "update:open", value: boolean): void;
  function emit(event: "update:title", value: string): void;
  function emit(event: string, value: unknown): void {
    emitted.push([event, value]);
  }
  const { open, title } = useVModels(props, emit, { keys: ["open", "title"] });

  open.value = true;
  title.value = "y";
  assert.deepEqual(emitted, [
    ["update:open", true],
    ["update:title", "y"],
  ]);
});
