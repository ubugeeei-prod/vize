import assert from "node:assert/strict";
import { test } from "node:test";
import { defineComponent, h, shallowRef } from "vue";

import { mountInMemory, textContent } from "./testing/memory-renderer.ts";
import { useSupported } from "./use-supported.ts";

void test("reports false until mount, then evaluates the reactive check", async () => {
  const available = shallowRef(true);
  const renders: string[] = [];
  const { root, unmount } = mountInMemory(
    defineComponent({
      setup() {
        const supported = useSupported(() => available.value);
        return () => {
          const text = String(supported.value);
          renders.push(text);
          return h("span", text);
        };
      },
    }),
  );
  assert.deepEqual(renders, ["false"]);
  await Promise.resolve();
  assert.equal(textContent(root), "true");
  available.value = false;
  await Promise.resolve();
  assert.equal(textContent(root), "false");
  unmount();
});

void test("outside components it depends on a browser window", () => {
  assert.equal(useSupported(() => true).value, false);
});
