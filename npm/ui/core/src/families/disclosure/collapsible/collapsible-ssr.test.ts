import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import CollapsibleContent from "./collapsible-content.vue";
import CollapsibleRoot from "./collapsible-root.vue";
import CollapsibleTrigger from "./collapsible-trigger.vue";

const SsrProbe = defineComponent({
  name: "CollapsibleSsrProbe",
  setup() {
    return () =>
      h(CollapsibleRoot, null, () => [
        h(CollapsibleTrigger, null, () => "Filters"),
        h(CollapsibleContent, null, () => "Filter controls"),
      ]);
  },
});

test("renders byte-identical closed disclosure markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /id="vize-v-\d+-collapsible"/);
  assert.match(html, /data-vize-ui="collapsible-root"/);
  assert.match(html, /data-state="closed"/);
  assert.match(html, /id="vize-v-\d+-collapsible-trigger"/);
  assert.match(html, /aria-expanded="false"/);
  assert.match(html, /aria-controls="vize-v-\d+-collapsible-content"/);
  assert.match(html, /id="vize-v-\d+-collapsible-content"/);
  assert.match(html, /role="region"/);
  assert.match(html, /hidden/);
  assert.match(html, /aria-labelledby="vize-v-\d+-collapsible-trigger"/);
});

test("hydrates generated ids without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverTrigger = host.querySelector<HTMLButtonElement>(
    "[data-vize-ui='collapsible-trigger']",
  );
  const serverContent = host.querySelector<HTMLDivElement>("[data-vize-ui='collapsible-content']");

  assert.ok(serverRoot);
  assert.ok(serverTrigger);
  assert.ok(serverContent);
  const triggerId = serverTrigger.id;
  const contentId = serverContent.id;
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
    const hydratedTrigger = host.querySelector<HTMLButtonElement>(
      "[data-vize-ui='collapsible-trigger']",
    );
    const hydratedContent = host.querySelector<HTMLDivElement>(
      "[data-vize-ui='collapsible-content']",
    );
    assert.ok(hydratedTrigger);
    assert.ok(hydratedContent);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydratedTrigger.id, triggerId);
    assert.equal(hydratedTrigger.getAttribute("aria-controls"), contentId);
    assert.equal(hydratedContent.id, contentId);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders open server markup without hidden content", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "OpenCollapsibleSsrProbe",
      setup() {
        return () =>
          h(CollapsibleRoot, { defaultOpen: true, id: "account-nav" }, () => [
            h(CollapsibleTrigger, null, () => "Account"),
            h(CollapsibleContent, { ariaDescribedby: "account-help" }, () =>
              h("p", { id: "account-help" }, "Account links"),
            ),
          ]);
      },
    }),
  );

  assert.match(html, /id="account-nav"/);
  assert.match(html, /id="account-nav-trigger"/);
  assert.match(html, /id="account-nav-content"/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /aria-describedby="account-help"/);
  assert.doesNotMatch(html, /hidden/);
});
