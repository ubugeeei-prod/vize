import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useSortable } from "./sortable.ts";

const SsrProbe = defineComponent({
  name: "SortableSsrProbe",
  setup() {
    const first = ref<HTMLElement | null>(null);
    const second = ref<HTMLElement | null>(null);
    const order = ref<readonly string[]>(["alpha", "bravo"]);
    const controller = useSortable({
      onSortCommit(event) {
        const next = [...order.value];
        const [moved] = next.splice(event.fromIndex, 1);
        if (moved !== undefined) next.splice(event.toIndex, 0, moved);
        order.value = next;
      },
    });
    const alpha = controller.registerItem({ key: "alpha", element: first });
    const bravo = controller.registerItem({ key: "bravo", element: second });
    return () =>
      h("ul", { "data-sorting": String(controller.isSorting.value) }, [
        h("li", { ...alpha.itemProps, ref: first, tabindex: 0 }, order.value.join(",")),
        h("li", { ...bravo.itemProps, ref: second, tabindex: 0 }, "bravo"),
      ]);
  },
});

test("renders byte-identical sortable SSR output without DOM access or handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<ul data-sorting="false"><li tabindex="0">alpha,bravo</li>' +
      '<li tabindex="0">bravo</li></ul>',
  );
  assert.doesNotMatch(outputs[0], /keydown|function|drag-and-drop-live/);
});

test("hydrates sortable state and reorders through a keyboard sort", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverList = host.querySelector("ul");
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.querySelector("ul"), serverList);
    const item = host.querySelector("li");
    assert.ok(item);
    const press = (key: string) =>
      item.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key }));
    press("Enter");
    await nextTick();
    assert.equal(host.querySelector("ul")?.getAttribute("data-sorting"), "true");
    press("ArrowDown");
    press("Enter");
    await nextTick();
    assert.equal(host.querySelector("ul")?.getAttribute("data-sorting"), "false");
    assert.equal(item.textContent, "bravo,alpha", "the committed sort must reorder");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    for (const region of document.querySelectorAll('[data-vize-ui="drag-and-drop-live"]')) {
      region.remove();
    }
    console.warn = originalWarn;
    console.error = originalError;
  }
});
