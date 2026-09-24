import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import HoverCardContent from "./hover-card-content.vue";
import HoverCardRoot from "./hover-card-root.vue";
import HoverCardTrigger from "./hover-card-trigger.vue";

function createProbe(rootProps: Record<string, unknown> = {}) {
  return defineComponent({
    name: "HoverCardSsrProbe",
    setup() {
      return () =>
        h("p", null, [
          "Written by ",
          h(HoverCardRoot, rootProps, () => [
            h(HoverCardTrigger, { href: "/users/ada" }, () => "@ada"),
            h(HoverCardContent, null, () => "Ada Lovelace"),
          ]),
        ]);
    },
  });
}

test("renders byte-identical closed hover-card markup that nests in phrasing content", async () => {
  const Probe = createProbe();
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /<span id="vize-v-\d+-hover-card"[^>]*data-vize-ui="hover-card-root"/);
  assert.match(html, /<a id="vize-v-\d+-hover-card-trigger"[^>]*href="\/users\/ada"/);
  assert.match(html, /<span[^>]*data-vize-ui="hover-card-content-host"[^>]*hidden/);
  assert.doesNotMatch(html, /<div/);
  assert.doesNotMatch(html, /aria-describedby/);
});

test("renders default-open content in place and hydrates without diagnostics", async () => {
  const Probe = defineComponent({
    name: "HoverCardOpenSsrProbe",
    setup() {
      return () =>
        h(HoverCardRoot, { defaultOpen: true, id: "card" }, () => [
          h(HoverCardTrigger, { href: "/users/ada" }, () => "@ada"),
          h(HoverCardContent, null, () => "Ada Lovelace"),
        ]);
    },
  });
  const serverHtml = await renderToString(createSSRApp(Probe));
  assert.match(serverHtml, /aria-describedby="card-content"/);
  assert.match(serverHtml, /id="card-content"/);

  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTrigger = host.querySelector("#card-trigger");
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
    assert.ok(host.querySelector("#card-trigger") === serverTrigger);
    assert.deepEqual(diagnostics, []);
    assert.ok(document.getElementById("card-content"));
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
