import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { ToastProvider, ToastViewport, useToast } from "./toast.ts";

const Seed = defineComponent({
  name: "ToastSsrSeed",
  setup() {
    const store = useToast();
    store.toast({
      title: "Welcome back",
      description: "3 unread messages",
      action: { label: "Open", altText: "Open the inbox from the sidebar" },
    });
    store.error({ title: "Sync failed", priority: "high" });
    return () => null;
  },
});

const SsrProbe = defineComponent({
  name: "ToastSsrProbe",
  setup() {
    return () => h(ToastProvider, { duration: 1000 }, () => [h(Seed), h(ToastViewport)]);
  },
});

test("renders byte-identical toast markup across isolated SSR requests without timers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="toast-provider"/);
  assert.match(html, /<section[^>]*aria-label="Notifications \(F8\)"/);
  assert.match(html, /tabindex="-1"/);
  assert.match(html, /<ol data-vize-ui="toast-list"/);
  assert.match(html, /<li[^>]*role="status"[^>]*aria-live="off"/);
  assert.match(html, /data-type="error"/);
  assert.match(html, /data-priority="high"/);
  assert.match(html, /Welcome back/);
  assert.match(html, /data-alt-text="Open the inbox from the sidebar"/);
  assert.match(html, /data-vize-ui="live-region"/);
});

test("hydrates server toasts without mismatches and announces after mount", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverItems = [...host.querySelectorAll("[data-vize-ui='toast']")];
  assert.equal(serverItems.length, 2);
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
    const hydratedItems = [...host.querySelectorAll("[data-vize-ui='toast']")];
    assert.equal(hydratedItems.length, 2);
    assert.ok(hydratedItems[0] === serverItems[0]);
    assert.ok(hydratedItems[1] === serverItems[1]);
    await nextTick();
    await nextTick();
    const region = host.querySelector("[data-vize-ui='live-region']");
    assert.equal(region?.getAttribute("aria-live"), "assertive");
    assert.match(region?.textContent ?? "", /Welcome back\. 3 unread messages/);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
