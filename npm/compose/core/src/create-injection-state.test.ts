import assert from "node:assert/strict";
import { test } from "node:test";
import { createApp, defineComponent, h, shallowRef } from "vue";
import type { InjectionKey, ShallowRef } from "vue";

import { createInjectionState } from "./create-injection-state.ts";
import { mountInMemory, textContent } from "./testing/memory-renderer.ts";

void test("provides to descendants and injects the nearest state", () => {
  const [useProvideCounter, useCounter] = createInjectionState((initial: number) => {
    const count = shallowRef(initial);
    return { count };
  });
  const seen: (number | undefined)[] = [];

  const Child = defineComponent({
    setup() {
      const counter = useCounter();
      seen.push(counter?.count.value);
      return () => h("span", String(counter?.count.value));
    },
  });
  const Parent = defineComponent({
    setup() {
      useProvideCounter(3);
      return () => h(Child);
    },
  });
  const Orphan = defineComponent({
    setup() {
      seen.push(useCounter()?.count.value);
      return () => null;
    },
  });

  const { root, unmount } = mountInMemory(
    defineComponent({ setup: () => () => [h(Parent), h(Orphan)] }),
  );
  assert.equal(textContent(root), "3");
  assert.deepEqual(seen, [3, undefined]);
  unmount();
});

void test("custom keys and default values", () => {
  const key: InjectionKey<ShallowRef<string>> = Symbol("theme");
  const fallback = shallowRef("light");
  const [, useTheme, returnedKey] = createInjectionState(() => shallowRef("dark"), {
    injectionKey: key,
    defaultValue: fallback,
  });
  assert.equal(returnedKey, key);

  const app = createApp({});
  assert.equal(
    app.runWithContext(() => useTheme()),
    fallback,
  );
  app.provide(key, shallowRef("dark"));
  assert.equal(
    app.runWithContext(() => useTheme().value),
    "dark",
  );
});
