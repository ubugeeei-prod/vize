import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import FloatingActionButton from "./floating-action-button.vue";
import SpeedDialAction from "./speed-dial-action.vue";
import SpeedDialContent from "./speed-dial-content.vue";
import SpeedDialRoot from "./speed-dial-root.vue";
import SpeedDialTrigger from "./speed-dial-trigger.vue";

const Probe = defineComponent({
  name: "SpeedDialSsrProbe",
  setup: () => () => [
    h(FloatingActionButton, { ariaLabel: "Compose" }, () => "+"),
    h(SpeedDialRoot, null, () => [
      h(SpeedDialTrigger, { ariaLabel: "Create" }, () => "+"),
      h(SpeedDialContent, null, () => [
        h(SpeedDialAction, { value: "note", label: "New note" }, () => "N"),
        h(SpeedDialAction, { value: "task", label: "New task" }, () => "T"),
      ]),
    ]),
  ],
});

test("renders byte-identical closed speed-dial markup", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="floating-action-button"/);
  assert.match(html, /id="vize-v-\d+-speed-dial-trigger"/);
  assert.match(html, /aria-expanded="false"/);
  assert.match(html, /role="menu"[^>]*hidden|hidden[^>]*role="menu"/);
  assert.match(html, /role="menuitem"/);
});

test("hydrates speed-dial markup without diagnostics", async () => {
  const host = document.createElement("div");
  host.innerHTML = await renderToString(createSSRApp(Probe));
  document.body.append(host);
  const serverTrigger = host.querySelector('[data-vize-ui="speed-dial-trigger"]');
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    assert.ok(host.querySelector('[data-vize-ui="speed-dial-trigger"]') === serverTrigger);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
