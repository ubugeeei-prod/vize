import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import InfiniteScrollItem from "./infinite-scroll-item.vue";
import InfiniteScrollLoadMore from "./infinite-scroll-load-more.vue";
import InfiniteScrollRoot from "./infinite-scroll-root.vue";
import InfiniteScrollSentinel from "./infinite-scroll-sentinel.vue";
import InfiniteScrollStatus from "./infinite-scroll-status.vue";

const SsrProbe = defineComponent({
  name: "InfiniteScrollSsrProbe",
  setup: () => () =>
    h(InfiniteScrollRoot, { feed: true, ariaLabel: "Timeline", total: 40 }, () => [
      h(InfiniteScrollItem, { index: 0 }, () => "First post"),
      h(InfiniteScrollItem, { index: 1 }, () => "Second post"),
      h(InfiniteScrollSentinel),
      h(InfiniteScrollLoadMore, null, () => "Load more"),
      h(InfiniteScrollStatus),
    ]),
});

test("renders byte-identical infinite scroll markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  const html = left;

  assert.match(html, /^<div id="vize-v-\d+-infinite-scroll" role="feed"/);
  assert.match(html, /aria-busy="false"/);
  assert.match(html, /aria-posinset="1" aria-setsize="40"/);
  assert.match(html, /aria-posinset="2" aria-setsize="40"/);
  assert.match(html, /data-vize-ui="infinite-scroll-sentinel"/);
  assert.match(html, /aria-controls="vize-v-\d+-infinite-scroll"/);
  assert.match(html, /role="status" aria-live="polite"/);
});

test("hydrates infinite scroll markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverId = serverRoot?.id;
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
    assert.equal(host.firstElementChild?.id, serverId);
    assert.equal(host.querySelectorAll("article").length, 2);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
