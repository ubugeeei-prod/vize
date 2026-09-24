import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import TocItem from "./toc-item.vue";
import TocLink from "./toc-link.vue";
import TocList from "./toc-list.vue";
import TocRoot from "./toc-root.vue";

const Probe = defineComponent({
  name: "TocSsrProbe",
  setup: () => () =>
    h(TocRoot, { defaultActiveId: "api" }, () =>
      h(TocList, null, () =>
        ["intro", "api"].map((id) =>
          h(TocItem, { key: id, targetId: id }, () => h(TocLink, { targetId: id }, () => id)),
        ),
      ),
    ),
});

test("renders byte-identical TOC markup with the default active section", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<nav/);
  assert.match(html, /aria-label="Table of contents"/);
  assert.match(html, /href="#api"[^>]*aria-current="location"/);
  assert.match(html, /data-vize-ui="toc-item" part="item" data-active="true"/);
});

test("hydrates the TOC without mismatch warnings", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
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
    assert.equal(host.querySelector('[aria-current="location"]')?.getAttribute("href"), "#api");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
