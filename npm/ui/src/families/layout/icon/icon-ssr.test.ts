import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Icon from "./icon.vue";

const SsrProbe = defineComponent({
  name: "IconSsrProbe",
  setup() {
    return () =>
      h(
        Icon,
        {
          description: "Reloads every dashboard panel",
          size: "sm",
          title: "Refresh panels",
        },
        {
          default: () => h("path", { d: "M4 12h16" }),
        },
      );
  },
});

test("renders byte-identical labelled SVG markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<svg/);
  assert.match(html, /role="img"/);
  assert.match(html, /data-vize-ui="icon"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-size="sm"/);
  assert.match(html, /data-aria-state="img"/);
  assert.match(html, /data-decorative="false"/);
  assert.match(html, /aria-labelledby="vize-v-[^"]+-icon-title"/);
  assert.match(html, /aria-describedby="vize-v-[^"]+-icon-description"/);
  assert.match(html, /<title id="vize-v-[^"]+-icon-title"[^>]*>Refresh panels<\/title>/);
  assert.match(
    html,
    /<desc id="vize-v-[^"]+-icon-description"[^>]*>Reloads every dashboard panel<\/desc>/,
  );
  assert.match(html, /<path d="M4 12h16"><\/path>/);
  assert.doesNotMatch(html, /class=|style=|tabindex=|aria-hidden=/);
});

test("hydrates labelled SVG markup without replacing the icon root", async () => {
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
    const hydrated = host.querySelector("[data-vize-ui='icon']");
    assert.ok(hydrated instanceof SVGSVGElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.getAttribute("role"), "img");
    assert.equal(hydrated.getAttribute("data-size"), "sm");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders decorative server markup without accessible semantics", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "IconDecorativeSsrProbe",
      setup() {
        return () =>
          h(
            Icon,
            { ariaHidden: true },
            {
              default: () => h("path", { d: "M2 12h20" }),
            },
          );
      },
    }),
  );

  assert.match(html, /^<svg/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-aria-state="decorative"/);
  assert.match(html, /data-title="missing"/);
  assert.match(html, /data-description="missing"/);
  assert.doesNotMatch(html, /role="img"/);
  assert.doesNotMatch(html, /aria-label=|aria-labelledby=|aria-describedby=/);
  assert.doesNotMatch(html, /<title|<desc/);
});
