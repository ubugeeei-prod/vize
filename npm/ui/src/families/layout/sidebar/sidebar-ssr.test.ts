import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import SidebarContent from "./sidebar-content.vue";
import SidebarInset from "./sidebar-inset.vue";
import SidebarProvider from "./sidebar-provider.vue";
import SidebarRoot from "./sidebar-root.vue";
import SidebarTrigger from "./sidebar-trigger.vue";
import type { SidebarStorage } from "./sidebar-types.ts";

function createProbe(props: Record<string, unknown> = {}) {
  return defineComponent({
    name: "SidebarSsrProbe",
    setup() {
      return () =>
        h(SidebarProvider, props, () => [
          h(SidebarRoot, null, () => h(SidebarContent, null, () => "Navigation")),
          h(SidebarInset, null, () => [h(SidebarTrigger), "Main"]),
        ]);
    },
  });
}

test("renders byte-identical desktop markup across isolated SSR requests", async () => {
  const Probe = createProbe({ defaultOpen: false });
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="sidebar-provider"/);
  assert.match(html, /--vize-sidebar-width:16rem/);
  assert.match(html, /<aside id="vize-v-\d+-sidebar-panel"[^>]*aria-label="Sidebar"[^>]*inert/);
  assert.match(html, /data-state="collapsed"/);
  assert.match(html, /aria-controls="vize-v-\d+-sidebar-panel"/);
  assert.match(html, /<main[^>]*data-vize-ui="sidebar-inset"/);
  assert.doesNotMatch(html, /data-mobile/);
});

test("hydrates without mismatches before applying storage and the mobile query", async () => {
  const storage: SidebarStorage = { get: () => "false", set: () => undefined };
  const Probe = createProbe({ id: "shell", storage });
  const serverHtml = await renderToString(createSSRApp(Probe));
  assert.match(serverHtml, /data-state="expanded"/);
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverAside = host.querySelector("aside");
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
    await nextTick();
    assert.deepEqual(diagnostics, []);
    assert.ok(host.querySelector("aside") === serverAside);
    assert.equal(host.querySelector("aside")?.getAttribute("data-state"), "collapsed");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
