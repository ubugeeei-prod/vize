import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useDragAndDrop } from "./drag-and-drop.ts";
import { pointer } from "./families/interaction/move/move-test-utils.ts";

const SsrProbe = defineComponent({
  name: "DragAndDropSsrProbe",
  setup() {
    const handle = ref<HTMLElement | null>(null);
    const zone = ref<HTMLElement | null>(null);
    const controller = useDragAndDrop();
    const source = controller.registerSource({
      key: "card",
      element: handle,
      payload: { kind: "card", data: 1 },
    });
    const target = controller.registerTarget({
      key: "zone",
      element: zone,
      getRect: () => ({ top: 0, left: 0, bottom: 100, right: 100 }),
    });
    return () =>
      h("div", [
        h("button", {
          ...source.sourceProps,
          ref: handle,
          type: "button",
          "data-dragging": String(source.isDragging.value),
        }),
        h("section", { ref: zone, "data-over": String(target.isOver.value) }),
      ]);
  },
});

test("renders byte-identical drag SSR output without DOM access or handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div><button type="button" data-dragging="false"></button>' +
      '<section data-over="false"></section></div>',
  );
  assert.doesNotMatch(outputs[0], /pointerdown|function|drag-and-drop-live/);
});

test("hydrates drag state without replacement or diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverButton = host.querySelector("button");
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.querySelector("button"), serverButton);
    const button = host.querySelector("button");
    assert.ok(button);
    button.dispatchEvent(pointer("pointerdown", 200, 200));
    document.dispatchEvent(pointer("pointermove", 50, 50));
    await nextTick();
    assert.equal(button.getAttribute("data-dragging"), "true");
    assert.equal(host.querySelector("section")?.getAttribute("data-over"), "true");
    document.dispatchEvent(pointer("pointerup", 50, 50));
    await nextTick();
    assert.equal(button.getAttribute("data-dragging"), "false");
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
