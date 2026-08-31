import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import PositionerArrow from "./positioner-arrow.vue";
import Positioner from "./positioner.vue";

const SsrProbe = defineComponent({
  name: "PositionerSsrProbe",
  setup() {
    return () =>
      h(Positioner, null, {
        default: () => [h(PositionerArrow), "Menu"],
      });
  },
});

test("renders byte-identical origin positioner SSR output without viewport reads", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /data-vize-ui="positioner"/);
  assert.match(outputs[0], /data-vize-positioner-ready="false"/);
  assert.match(outputs[0], /data-vize-placement="bottom"/);
  assert.match(outputs[0], /data-vize-ui="positioner-arrow"/);
  assert.match(outputs[0], /Menu/);
});

test("hydrates the positioner without replacement or diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTarget = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverTarget);
    await nextTick();
    assert.equal(host.firstElementChild?.getAttribute("data-vize-ui"), "positioner");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
