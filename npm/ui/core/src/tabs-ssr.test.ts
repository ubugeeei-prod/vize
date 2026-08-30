import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import TabsContent from "./tabs-content.vue";
import TabsList from "./tabs-list.vue";
import TabsRoot from "./tabs-root.vue";
import TabsTrigger from "./tabs-trigger.vue";

const SsrProbe = defineComponent({
  name: "TabsSsrProbe",
  setup: () => () =>
    h(TabsRoot, { defaultValue: "overview" }, () => [
      h(TabsList, { ariaLabel: "Product sections" }, () => [
        h(TabsTrigger, { value: "overview" }, () => "Overview"),
        h(TabsTrigger, { value: "details" }, () => "Details"),
      ]),
      h(TabsContent, { value: "overview" }, () => "Overview panel"),
      h(TabsContent, { value: "details" }, () => "Details panel"),
    ]),
});

test("renders byte-identical tabs markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /id="vize-v-\d+-tabs"/);
  assert.match(html, /role="tablist"/);
  assert.match(html, /aria-label="Product sections"/);
  assert.match(html, /id="vize-v-\d+-tabs-trigger-value-overview"/);
  assert.match(html, /aria-selected="true"/);
  assert.match(html, /aria-controls="vize-v-\d+-tabs-content-value-overview"/);
  assert.match(html, /id="vize-v-\d+-tabs-content-value-overview"/);
  assert.match(html, /role="tabpanel"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /id="vize-v-\d+-tabs-content-value-details"[^>]*hidden/);
});

test("hydrates generated tabs ids without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverTriggers = [...host.querySelectorAll<HTMLButtonElement>("[role='tab']")];
  const serverPanels = [...host.querySelectorAll<HTMLDivElement>("[role='tabpanel']")];
  assert.ok(serverRoot);
  assert.equal(serverTriggers.length, 2);
  assert.equal(serverPanels.length, 2);
  const triggerIds = serverTriggers.map((trigger) => trigger.id);
  const panelIds = serverPanels.map((panel) => panel.id);

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
    const hydratedTriggers = [...host.querySelectorAll<HTMLButtonElement>("[role='tab']")];
    const hydratedPanels = [...host.querySelectorAll<HTMLDivElement>("[role='tabpanel']")];
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      hydratedTriggers.map((trigger) => trigger.id),
      triggerIds,
    );
    assert.deepEqual(
      hydratedPanels.map((panel) => panel.id),
      panelIds,
    );
    assert.equal(hydratedTriggers[0]?.getAttribute("aria-selected"), "true");
    assert.equal(hydratedPanels[1]?.hidden, true);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
