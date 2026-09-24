import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import NavigationMenuContent from "./navigation-menu-content.vue";
import NavigationMenuIndicator from "./navigation-menu-indicator.vue";
import NavigationMenuItem from "./navigation-menu-item.vue";
import NavigationMenuLink from "./navigation-menu-link.vue";
import NavigationMenuList from "./navigation-menu-list.vue";
import NavigationMenuRoot from "./navigation-menu-root.vue";
import NavigationMenuTrigger from "./navigation-menu-trigger.vue";
import NavigationMenuViewport from "./navigation-menu-viewport.vue";

const Probe = defineComponent({
  name: "NavigationMenuSsrProbe",
  setup: () => () =>
    h(NavigationMenuRoot, { ariaLabel: "Main", defaultValue: "learn" }, () => [
      h(NavigationMenuList, null, () => [
        h(NavigationMenuItem, { value: "learn" }, () => [
          h(NavigationMenuTrigger, null, () => "Learn"),
          h(NavigationMenuContent, null, () =>
            h(NavigationMenuLink, { href: "/guide" }, () => "Guide"),
          ),
        ]),
        h(NavigationMenuItem, { value: "api" }, () => [
          h(NavigationMenuTrigger, null, () => "API"),
          h(NavigationMenuContent, { forceMount: true }, () =>
            h(NavigationMenuLink, { href: "/api" }, () => "Reference"),
          ),
        ]),
        h(NavigationMenuIndicator),
      ]),
      h(NavigationMenuViewport),
    ]),
});

test("renders byte-identical navigation menus with the default flyout open", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<nav/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /id="vize-v-\d+-navigation-menu-content-value-learn"/);
  assert.match(html, /id="vize-v-\d+-navigation-menu-content-value-api"[^>]*hidden/);
  assert.doesNotMatch(html, /--vize-navigation-menu-indicator/, "geometry is client-only");
});

test("hydrates the navigation menu without mismatch warnings", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
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
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(host.querySelector("button")?.getAttribute("aria-expanded"), "true");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
