import assert from "node:assert/strict";
import { test } from "node:test";
import { defineComponent, h } from "vue";

import { mountInMemory, textContent } from "./testing/memory-renderer.ts";
import { useMounted } from "./use-mounted.ts";

void test("is false during setup and render, true after mount", async () => {
  const snapshots: boolean[] = [];
  const { root, unmount } = mountInMemory(
    defineComponent({
      setup() {
        const mounted = useMounted();
        snapshots.push(mounted.value);
        return () => h("span", mounted.value ? "client" : "fallback");
      },
    }),
  );
  assert.deepEqual(snapshots, [false]);
  await Promise.resolve();
  assert.equal(textContent(root), "client");
  unmount();
});

void test("stays false outside a component", () => {
  assert.equal(useMounted().value, false);
});
