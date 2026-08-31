import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import type { Ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { createCollectionRegistry } from "../collection/collection.ts";
import { useCompositeNavigation } from "./composite-navigation.ts";

const CompositeSsrProbe = defineComponent({
  name: "CompositeSsrProbe",
  setup() {
    const registry = createCollectionRegistry<string, string>();
    const elements: Record<string, Ref<Element | null>> = {
      alpha: ref(null),
      bravo: ref(null),
    };
    for (const [index, key] of ["alpha", "bravo"].entries()) {
      registry.register({
        key,
        value: key,
        textValue: key,
        order: index,
        element: () => elements[key]?.value,
      });
    }
    const navigation = useCompositeNavigation({
      registry,
      focusStrategy: "active-descendant",
      getItemId: ({ key }) => `option-${key}`,
    });
    return () =>
      h(
        "div",
        {
          ...navigation.getContainerProps(),
          role: "listbox",
          "data-active": navigation.activeKey.value ?? "none",
        },
        ["alpha", "bravo"].map((key) =>
          h(
            "div",
            {
              ...navigation.getItemProps(key),
              ref: elements[key],
              role: "option",
              "aria-selected": navigation.activeKey.value === key,
            },
            key,
          ),
        ),
      );
  },
});

test("renders byte-identical active-descendant markup without serializing handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(CompositeSsrProbe)),
    renderToString(createSSRApp(CompositeSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div tabindex="0" aria-activedescendant="option-alpha" role="listbox" data-active="none"><div id="option-alpha" role="option" aria-selected="false">alpha</div><div id="option-bravo" role="option" aria-selected="false">bravo</div></div>',
  );
  assert.doesNotMatch(outputs[0], /onFocus|onKeydown|function/);
});

test("hydrates in place and navigates without mismatch diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(CompositeSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverContainer = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(CompositeSsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverContainer);
    const container = host.firstElementChild as HTMLElement;
    container.focus();
    container.dispatchEvent(
      new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "ArrowDown" }),
    );
    await nextTick();
    assert.equal(container.dataset.active, "bravo");
    assert.equal(container.getAttribute("aria-activedescendant"), "option-bravo");
    assert.equal(document.activeElement, container);
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
