import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ImageCompareAfter from "./image-compare-after.vue";
import ImageCompareBefore from "./image-compare-before.vue";
import ImageCompareHandle from "./image-compare-handle.vue";
import ImageCompareLabel from "./image-compare-label.vue";
import ImageCompareRoot from "./image-compare-root.vue";

const SsrProbe = defineComponent({
  name: "ImageCompareSsrProbe",
  setup: () => () =>
    h(ImageCompareRoot, { defaultValue: 35, orientation: "vertical" }, () => [
      h(ImageCompareBefore, null, () => h("img", { alt: "Before", src: "/before.jpg" })),
      h(ImageCompareAfter, null, () => h("img", { alt: "After", src: "/after.jpg" })),
      h(ImageCompareLabel, { side: "before" }, () => "Before"),
      h(ImageCompareHandle),
    ]),
});

test("renders byte-identical image compare markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(
    left,
    /^<div id="vize-v-\d+-image-compare" dir="ltr" data-vize-ui="image-compare-root"/,
  );
  assert.match(left, /--vize-ui-image-compare-position:35%/);
  assert.match(left, /id="vize-v-\d+-image-compare-handle" role="slider" tabindex="0"/);
  assert.match(left, /aria-orientation="vertical"/);
  assert.match(left, /aria-valuenow="35" aria-valuetext="35%"/);
  assert.match(left, /data-vize-ui="image-compare-label" part="label" data-side="before"/);
});

test("hydrates image compare markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverHandle = host.querySelector('[role="slider"]');
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
    assert.ok(host.querySelector('[role="slider"]') === serverHandle);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
