import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import LandmarkProvider from "./landmark-provider.vue";
import Landmark from "./landmark.vue";

const Page = defineComponent({
  name: "LandmarkSsrPage",
  setup: () => () =>
    h("div", null, [
      h(LandmarkProvider, { discover: true }, () => [
        h(Landmark, { role: "banner" }, () => "Top"),
        h(Landmark, { role: "navigation", ariaLabel: "Primary" }, () => "Nav"),
        h(Landmark, { role: "main" }, () => "Content"),
        h(Landmark, { role: "search", ariaLabel: "Site" }, () => "Search"),
      ]),
    ]),
});

test("renders byte-identical native landmark markup across SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Page)),
    renderToString(createSSRApp(Page)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /<header id="vize-v-\d+-landmark"[^>]*data-landmark="banner"/);
  assert.match(html, /<nav[^>]*aria-label="Primary"/);
  assert.match(html, /<main[^>]*data-vize-ui="landmark"/);
  assert.match(html, /<search[^>]*aria-label="Site"/);
  assert.doesNotMatch(html, /tabindex/);
});

test("hydrates landmarks without diagnostics and cycles only after mount", async () => {
  const host = document.createElement("div");
  host.innerHTML = await renderToString(createSSRApp(Page));
  document.body.append(host);
  const serverMain = host.querySelector("main");
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Page);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    assert.ok(host.querySelector("main") === serverMain);
    assert.deepEqual(diagnostics, []);
    document.body.dispatchEvent(
      new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "F6" }),
    );
    assert.equal(document.activeElement?.localName, "header");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
