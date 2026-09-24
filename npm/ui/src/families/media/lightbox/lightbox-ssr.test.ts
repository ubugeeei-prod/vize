import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import type { LightboxSlotState } from "./lightbox.ts";
import LightboxClose from "./lightbox-close.vue";
import LightboxContent from "./lightbox-content.vue";
import LightboxCounter from "./lightbox-counter.vue";
import LightboxImage from "./lightbox-image.vue";
import LightboxItem from "./lightbox-item.vue";
import LightboxNext from "./lightbox-next.vue";
import LightboxPrevious from "./lightbox-previous.vue";
import LightboxRoot from "./lightbox-root.vue";
import LightboxThumbnail from "./lightbox-thumbnail.vue";
import LightboxThumbnails from "./lightbox-thumbnails.vue";
import LightboxTrigger from "./lightbox-trigger.vue";

const photos = ["/a.jpg", "/b.jpg"];

function probe(defaultOpen: boolean) {
  return defineComponent({
    name: "LightboxSsrProbe",
    setup: () => () =>
      h(
        LightboxRoot,
        { items: photos, defaultOpen, defaultIndex: 1 },
        {
          default: (state: LightboxSlotState<string>) => [
            h(LightboxTrigger, { index: 0 }, () => "Open first"),
            h(LightboxContent, { portalDisabled: true }, () => [
              h(LightboxItem, null, () =>
                h(LightboxImage, { src: state.item ?? "", alt: "Photo" }),
              ),
              h(LightboxPrevious),
              h(LightboxNext),
              h(LightboxClose),
              h(LightboxCounter),
              h(LightboxThumbnails, null, () => [
                h(LightboxThumbnail, { index: 0 }),
                h(LightboxThumbnail, { index: 1 }),
              ]),
            ]),
          ],
        },
      ),
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

test("renders byte-identical closed and open lightbox markup across isolated SSR requests", async () => {
  const closed = await renderTwice(probe(false));
  assert.match(
    closed,
    /^<div data-vize-ui="lightbox-root" part="root" dir="ltr" data-state="closed"/,
  );
  assert.match(closed, /aria-haspopup="dialog" aria-expanded="false"/);
  assert.doesNotMatch(closed, /data-vize-ui="lightbox-content"/);

  const open = await renderTwice(probe(true));
  assert.match(open, /data-state="open"/);
  assert.match(open, /role="dialog"/);
  assert.match(open, /aria-label="Media viewer"/);
  assert.match(open, /id="vize-v-\d+-lightbox-item-1"/);
  assert.match(open, /aria-label="2 of 2"/);
  assert.match(open, /src="\/b\.jpg"/);
  assert.match(open, /role="tablist"/);
  assert.match(open, /aria-live="polite"/);
});

test("hydrates open lightbox markup without warnings or node replacement", async () => {
  const component = probe(true);
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
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
