import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import MediaPreferencesProvider from "./media-preferences-provider.vue";
import { usePrefersReducedMotion } from "./media-preferences-runtime.ts";

const Reader = defineComponent({
  name: "MediaPreferencesSsrReader",
  setup() {
    const motion = usePrefersReducedMotion();
    return () => h("span", motion.value ? "calm" : "lively");
  },
});

const Probe = defineComponent({
  name: "MediaPreferencesSsrProbe",
  setup: () => () =>
    h(MediaPreferencesProvider, { initial: { colorScheme: "dark" } }, () => h(Reader)),
});

test("renders byte-identical markup from defaults and initial hints", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="media-preferences-provider"/);
  assert.match(html, /data-color-scheme="dark"/);
  assert.match(html, /data-prefers-contrast="no-preference"/);
  assert.doesNotMatch(html, /data-reduced-motion/);
  assert.match(html, /lively/);
});

test("hydrates with the server snapshot, then applies detected preferences", async () => {
  const originalMatchMedia = window.matchMedia;
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: (query: string) => ({
      addEventListener: () => undefined,
      matches: query === "(prefers-reduced-motion: reduce)",
      media: query,
      removeEventListener: () => undefined,
    }),
    writable: true,
  });
  const host = document.createElement("div");
  host.innerHTML = await renderToString(createSSRApp(Probe));
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
    assert.deepEqual(diagnostics, [], "hydration matches the server snapshot");
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(serverRoot?.getAttribute("data-reduced-motion"), "true");
    assert.equal(serverRoot?.textContent, "calm");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: originalMatchMedia,
      writable: true,
    });
  }
});
