import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ImageCropperArea from "./image-cropper-area.vue";
import ImageCropperGrid from "./image-cropper-grid.vue";
import ImageCropperHandle from "./image-cropper-handle.vue";
import ImageCropperImage from "./image-cropper-image.vue";
import ImageCropperRoot from "./image-cropper-root.vue";
import ImageCropperViewport from "./image-cropper-viewport.vue";

const SsrProbe = defineComponent({
  name: "ImageCropperSsrProbe",
  setup: () => () =>
    h(ImageCropperRoot, { aspectRatio: 1 }, () =>
      h(ImageCropperViewport, null, () => [
        h(ImageCropperImage, { src: "/portrait.jpg", alt: "Portrait" }),
        h(ImageCropperArea, null, () => [
          h(ImageCropperGrid),
          h(ImageCropperHandle, { position: "se" }),
        ]),
      ]),
    ),
});

test("renders byte-identical cropper markup without measured geometry on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(
    left,
    /^<div id="vize-v-\d+-image-cropper" data-vize-ui="image-cropper-root" part="root" data-interaction="idle"/,
  );
  assert.match(
    left,
    /data-vize-ui="image-cropper-viewport" part="viewport" style="--vize-ui-image-cropper-rotation:0deg;--vize-ui-image-cropper-scale:0;--vize-ui-image-cropper-zoom:1;overflow:hidden;position:relative;touch-action:none;"/,
  );
  assert.match(left, /<img alt="Portrait" src="\/portrait\.jpg" draggable="false"/);
  assert.match(left, /role="group" tabindex="0" aria-roledescription="crop area"/);
  assert.match(left, /data-vize-ui="image-cropper-handle" part="handle" data-position="se"/);
  assert.equal(left.match(/image-cropper-grid-line/g)?.length, 4);
  assert.doesNotMatch(left, /data-ready/);
});

test("hydrates cropper markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverImage = host.querySelector("img");
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
    assert.ok(host.querySelector("img") === serverImage);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
