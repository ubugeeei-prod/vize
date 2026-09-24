import assert from "node:assert/strict";
import { test } from "node:test";
import { defineComponent, h } from "vue";

import { mountInMemory } from "./testing/memory-renderer.ts";
import { useIdGenerator } from "./use-id-generator.ts";

void test("generates sequential sanitized ids from the seed", () => {
  const nextId = useIdGenerator({ prefix: "field", seed: "form:1" });
  assert.equal(nextId("input"), "field-form-1-input-0");
  assert.equal(nextId(), "field-form-1-id-1");
  assert.equal(nextId("hint text"), "field-form-1-hint-text-2");
  assert.equal(useIdGenerator()(), "vize-root-id-0");
});

void test("seeds from Vue's useId inside components", () => {
  const ids: string[] = [];
  const Field = defineComponent({
    setup() {
      const nextId = useIdGenerator();
      ids.push(nextId("label"), nextId("input"));
      return () => null;
    },
  });
  const { unmount } = mountInMemory(defineComponent({ setup: () => () => [h(Field), h(Field)] }));
  assert.equal(ids.length, 4);
  assert.equal(new Set(ids).size, 4);
  for (const id of ids) assert.match(id, /^vize-v-[\w-]+-(label|input)-[01]$/);
  unmount();
});
