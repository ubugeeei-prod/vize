import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import AudioVisualizer from "./audio-visualizer.vue";
import AudioVisualizerBars from "./audio-visualizer-bars.vue";

const SsrProbe = defineComponent({
  name: "AudioVisualizerSsrProbe",
  setup: () => () =>
    h(AudioVisualizer, { ariaLabel: "Output level", fftSize: 64 }, () =>
      h(AudioVisualizerBars, { count: 3 }),
    ),
});

test("renders byte-identical idle visualizer markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(
    left,
    /^<div role="img" aria-label="Output level" data-vize-ui="audio-visualizer-root"/,
  );
  assert.match(left, /data-state="idle"/);
  assert.match(left, /--vize-ui-audio-visualizer-level:0/);
  assert.equal(left.match(/data-vize-ui="audio-visualizer-bar"/g)?.length, 3);
  assert.match(left, /--vize-ui-audio-visualizer-bar:0/);
});

test("hydrates visualizer markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(host.querySelectorAll('[data-vize-ui="audio-visualizer-bar"]').length, 3);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
