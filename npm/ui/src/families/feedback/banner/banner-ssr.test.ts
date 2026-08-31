import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Banner from "./banner.vue";

const SsrProbe = defineComponent({
  name: "BannerSsrProbe",
  setup() {
    return () =>
      h(
        Banner,
        {
          description: "Scheduled from 02:00 to 02:15 UTC.",
          id: "maintenance-banner",
          title: "System maintenance",
          tone: "warning",
        },
        {
          default: () => h("p", "Some features will pause briefly."),
        },
      );
  },
});

test("renders byte-identical named region markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<section/);
  assert.match(html, /id="maintenance-banner"/);
  assert.match(html, /role="region"/);
  assert.match(html, /aria-labelledby="maintenance-banner-title"/);
  assert.match(html, /aria-describedby="maintenance-banner-description"/);
  assert.match(html, /data-vize-ui="banner"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="open"/);
  assert.match(html, /data-tone="warning"/);
  assert.match(html, /data-role="region"/);
  assert.match(html, /data-live="off"/);
  assert.match(html, /data-named="true"/);
  assert.match(html, /data-aria-state="named"/);
  assert.match(html, /id="maintenance-banner-title"/);
  assert.match(html, /id="maintenance-banner-description"/);
  assert.match(html, /System maintenance/);
  assert.match(html, /Some features will pause briefly/);
  assert.doesNotMatch(html, /class=|style=|tabindex=|function/);
  assert.doesNotMatch(html, /aria-live|aria-atomic/);
});

test("hydrates named SSR markup without replacing the banner root", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  assert.ok(serverRoot);
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
    const hydrated = host.querySelector<HTMLElement>("[data-vize-ui='banner']");
    assert.ok(hydrated);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.getAttribute("role"), "region");
    assert.equal(hydrated.getAttribute("aria-labelledby"), "maintenance-banner-title");
    assert.equal(hydrated.getAttribute("data-state"), "open");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders server live-role markup with deterministic title naming", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "BannerLiveSsrProbe",
      setup() {
        return () =>
          h(Banner, {
            atomic: false,
            id: "deploy-banner",
            role: "status",
            title: "Deploy status",
            tone: "info",
          });
      },
    }),
  );

  assert.match(html, /^<section/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-labelledby="deploy-banner-title"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /aria-atomic="false"/);
  assert.match(html, /data-live="polite"/);
  assert.match(html, /data-aria-state="live"/);
});

test("renders closed server markup as hidden decorative content", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "BannerClosedSsrProbe",
      setup() {
        return () => h(Banner, { open: false, title: "Closed update" });
      },
    }),
  );

  assert.match(html, /^<section/);
  assert.match(html, /hidden/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-state="closed"/);
  assert.match(html, /Closed update/);
  assert.doesNotMatch(html, /\srole="region"|aria-labelledby|aria-live|aria-atomic/);
});
