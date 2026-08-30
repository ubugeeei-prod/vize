import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import SkipLink from "./skip-link.vue";
import type { SkipLinkSlotState } from "./skip-link.ts";

const SsrProbe = defineComponent({
  name: "SkipLinkSsrProbe",
  setup() {
    return () =>
      h(
        SkipLink,
        { href: "#main", id: "skip-main" },
        {
          default: ({ targetId }: SkipLinkSlotState) => `Skip to ${targetId}`,
        },
      );
  },
});

test("renders byte-identical native anchor markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<a/);
  assert.match(html, /id="skip-main"/);
  assert.match(html, /href="#main"/);
  assert.match(html, /data-vize-ui="skip-link"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-target-id="main"/);
  assert.match(html, /Skip to main/);
  assert.match(html, /<\/a>$/);
  assert.doesNotMatch(html, /aria-disabled=/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
  assert.doesNotMatch(html, /tabindex=/);
});

test("hydrates without replacing the skip link root", async () => {
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
    const hydrated = host.querySelector<HTMLAnchorElement>("[data-vize-ui='skip-link']");
    assert.ok(hydrated);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.id, "skip-main");
    assert.equal(hydrated.getAttribute("href"), "#main");
    assert.equal(hydrated.getAttribute("data-state"), "idle");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
