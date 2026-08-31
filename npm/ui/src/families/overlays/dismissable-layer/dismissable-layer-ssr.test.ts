import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useDismissableLayer } from "./dismissable-layer.ts";

const DismissableLayerSsrProbe = defineComponent({
  name: "DismissableLayerSsrProbe",
  setup() {
    const root = ref<HTMLElement | null>(null);
    const lastDismissal = ref("none");
    const layer = useDismissableLayer({
      root,
      onDismiss(event) {
        lastDismissal.value = event.reason;
      },
    });
    return () =>
      h(
        "section",
        {
          ...layer.layerProps,
          "data-active": String(layer.isActive.value),
          "data-dismissed": lastDismissal.value,
          "data-top": String(layer.isTopLayer.value),
          ref: root,
        },
        h("button", { type: "button" }, "Inside"),
      );
  },
});

test("renders deterministic inactive markup without document access or serialized handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(DismissableLayerSsrProbe)),
    renderToString(createSSRApp(DismissableLayerSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0]!, /data-vize-dismissable-layer/);
  assert.match(outputs[0]!, /data-active="false"/);
  assert.match(outputs[0]!, /data-top="false"/);
  assert.doesNotMatch(outputs[0]!, /pointerdown|focusin|keydown|function/);
});

test("hydrates in place, activates after mount, and keeps dismissal state reactive", async () => {
  const serverHtml = await renderToString(createSSRApp(DismissableLayerSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(DismissableLayerSsrProbe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.equal(serverRoot?.getAttribute("data-active"), "true");
    assert.equal(serverRoot?.getAttribute("data-top"), "true");
    serverRoot?.dispatchEvent(
      new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
    );
    await nextTick();
    assert.equal(serverRoot?.getAttribute("data-dismissed"), "escape-key");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
