import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import MarqueeContent from "./marquee-content.vue";
import MarqueePauseButton from "./marquee-pause-button.vue";
import MarqueeRoot from "./marquee-root.vue";

const SsrProbe = defineComponent({
  name: "MarqueeSsrProbe",
  setup: () => () =>
    h(MarqueeRoot, { ariaLabel: "Sponsors", direction: "right" }, () => [
      h(MarqueePauseButton),
      h(MarqueeContent, null, () => "Acme · Globex · Initech"),
    ]),
});

test("renders byte-identical marquee markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /^<div id="vize-v-\d+-marquee" role="marquee" aria-label="Sponsors"/);
  assert.match(left, /style="--vize-ui-marquee-copies: 2"/);
  assert.match(left, /data-state="running"/);
  assert.match(left, /data-measured="false"/);
  assert.match(left, /aria-label="Pause scrolling content"/);
  assert.match(left, /data-copy="1" aria-hidden="true" inert/);
  assert.doesNotMatch(left, /--vize-ui-marquee-duration/);
});

test("hydrates marquee markup without warnings or node replacement", async () => {
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
    assert.equal(host.querySelectorAll('[data-vize-ui="marquee-content"]').length, 2);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
