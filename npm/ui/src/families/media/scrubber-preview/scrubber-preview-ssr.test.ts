import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ScrubberPreviewRoot from "./scrubber-preview-root.vue";
import ScrubberPreviewThumbnail from "./scrubber-preview-thumbnail.vue";
import ScrubberPreviewTime from "./scrubber-preview-time.vue";
import ScrubberPreviewTrack from "./scrubber-preview-track.vue";

function probe(props: Record<string, unknown>) {
  return defineComponent({
    name: "ScrubberPreviewSsrProbe",
    setup: () => () =>
      h(ScrubberPreviewRoot, { duration: 90, ...props }, () => [
        h(ScrubberPreviewTrack, null, () => h("div", { "data-bar": "" })),
        h(ScrubberPreviewThumbnail),
        h(ScrubberPreviewTime),
      ]),
  });
}

async function renderTwice(component: ReturnType<typeof probe>): Promise<string> {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(component)),
    renderToString(createSSRApp(component)),
  ]);
  assert.equal(left, right);
  return left;
}

test("renders byte-identical scrubber preview markup across isolated SSR requests", async () => {
  const idle = await renderTwice(probe({ videoSrc: "/movie.mp4" }));
  assert.match(
    idle,
    /^<div data-vize-ui="scrubber-preview-root" part="root" data-state="idle" data-kind="capture"/,
  );
  assert.match(idle, /data-vize-ui="scrubber-preview-track"/);
  assert.match(idle, /aria-hidden="true" hidden data-vize-ui="scrubber-preview-thumbnail"/);
  assert.doesNotMatch(idle, /<video|<img/);

  const sprite = await renderTwice(
    probe({
      time: 12,
      sprite: { src: "/sheet.jpg", columns: 4, rows: 4, interval: 5, width: 120, height: 68 },
    }),
  );
  assert.match(sprite, /data-state="active" data-kind="sprite"/);
  assert.match(sprite, /--vize-ui-scrubber-preview-ratio:0.13333/);
  assert.match(sprite, /--vize-ui-scrubber-preview-x:-240px/);
  assert.match(sprite, />(?:<!--\[-->)?0:12(?:<!--\]-->)?<\/span>/);
});

test("hydrates scrubber preview markup without warnings or node replacement", async () => {
  const component = probe({
    time: 30,
    thumbnails: "WEBVTT\n\n00:00.000 --> 01:00.000\n/t.jpg#xywh=0,0,10,10\n",
  });
  const serverHtml = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(
      host
        .querySelector('[data-vize-ui="scrubber-preview-thumbnail"]')
        ?.getAttribute("data-status"),
      "ready",
    );
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
