import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import PanZoomContent from "./pan-zoom-content.vue";
import PanZoomFit from "./pan-zoom-fit.vue";
import PanZoomReset from "./pan-zoom-reset.vue";
import PanZoomRoot from "./pan-zoom-root.vue";
import PanZoomStatus from "./pan-zoom-status.vue";
import PanZoomViewport from "./pan-zoom-viewport.vue";
import PanZoomZoomIn from "./pan-zoom-zoom-in.vue";
import PanZoomZoomOut from "./pan-zoom-zoom-out.vue";

const SsrProbe = defineComponent({
  name: "PanZoomSsrProbe",
  setup: () => () =>
    h(PanZoomRoot, { defaultValue: { x: 12, y: -8, scale: 2 }, maxScale: 2 }, () => [
      h(PanZoomViewport, { ariaLabel: "Map" }, () =>
        h(PanZoomContent, null, () => h("img", { alt: "Map", src: "/map.png" })),
      ),
      h(PanZoomZoomIn),
      h(PanZoomZoomOut),
      h(PanZoomReset),
      h(PanZoomFit),
      h(PanZoomStatus),
    ]),
});

test("renders byte-identical pan-zoom markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /^<div data-vize-ui="pan-zoom-root" part="root" data-state="idle"/);
  assert.match(
    left,
    /role="group" tabindex="0" aria-roledescription="pan and zoom area" aria-label="Map"/,
  );
  assert.match(left, /transform:translate\(12px, -8px\) scale\(2\)/);
  assert.match(left, /--vize-ui-pan-zoom-scale:2/);
  assert.match(left, /<button type="button" disabled data-vize-ui="pan-zoom-zoom-in"/);
  assert.match(left, /role="status" aria-live="polite" aria-atomic="true"[^>]*><!--\[-->200%/);
  assert.doesNotMatch(left, /touch-action/, "touch-action is applied on the client only");
});

test("hydrates pan-zoom markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverContent = host.querySelector('[data-vize-ui="pan-zoom-content"]');
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
    assert.ok(host.querySelector('[data-vize-ui="pan-zoom-content"]') === serverContent);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
