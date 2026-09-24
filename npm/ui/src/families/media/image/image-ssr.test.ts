import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ImageContent from "./image-content.vue";
import ImageFallback from "./image-fallback.vue";
import ImagePlaceholder from "./image-placeholder.vue";
import ImageRoot from "./image-root.vue";

function probe(rootProps: Record<string, unknown>, placeholderDelay = 0) {
  return defineComponent({
    name: "ImageSsrProbe",
    setup: () => () =>
      h(ImageRoot, rootProps, () => [
        h(ImageContent, { alt: "Harbor at dusk", width: 800, height: 600 }),
        h(ImagePlaceholder, { delay: placeholderDelay }, () => "Loading"),
        h(ImageFallback, null, () => "HD"),
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

test("renders byte-identical loading image markup across isolated SSR requests", async () => {
  const html = await renderTwice(probe({ src: ["javascript:alert(1)", "/harbor.webp"] }));

  assert.match(html, /^<span data-vize-ui="image-root" part="root" data-status="loading"/);
  assert.match(html, /<img alt="Harbor at dusk"/);
  assert.match(html, /src="\/harbor\.webp"/);
  assert.match(html, /loading="lazy"/);
  assert.match(html, /decoding="async"/);
  assert.match(html, /data-vize-ui="image-placeholder"/);
  assert.doesNotMatch(html, /javascript:/);
  assert.doesNotMatch(html, /image-fallback/);
});

test("renders fallback, idle, and delayed-placeholder states deterministically on the server", async () => {
  const failed = await renderTwice(probe({ src: [] }));
  assert.match(failed, /data-status="error"/);
  assert.match(failed, /data-vize-ui="image-fallback"/);
  assert.doesNotMatch(failed, /<img/);

  const deferred = await renderTwice(probe({ src: "/harbor.webp", defer: true }));
  assert.match(deferred, /data-status="idle"/);
  assert.match(deferred, /data-deferred="true"/);
  assert.doesNotMatch(deferred, /src=/);

  const delayed = await renderTwice(probe({ src: "/harbor.webp" }, 150));
  assert.doesNotMatch(delayed, /image-placeholder/);
});

test("hydrates server image markup without warnings or node replacement", async () => {
  const component = probe({ src: "/harbor.webp" });
  const serverHtml = await renderToString(createSSRApp(component));
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
  const app = createSSRApp(component);
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
