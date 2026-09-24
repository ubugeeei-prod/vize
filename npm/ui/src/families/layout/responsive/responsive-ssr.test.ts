import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import ResponsiveShow from "./responsive-show.vue";
import ResponsiveSwitch from "./responsive-switch.vue";

const Probe = defineComponent({
  name: "ResponsiveSsrProbe",
  setup: () => () =>
    h("main", [
      h(ResponsiveSwitch, { ssrWidth: 800 }, { base: () => "phone", md: () => "tablet" }),
      h(ResponsiveShow, { above: "lg", ssrWidth: 800, hideMode: "hidden" }, () => "desktop"),
      h(ResponsiveShow, { below: "lg", ssrWidth: 800 }, () => "compact"),
    ]),
});

test("renders byte-identical breakpoint markup from ssrWidth across requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-breakpoint="md" data-slot="md"[^>]*><!--\[-->tablet</);
  assert.match(html, /data-state="hidden" hidden[^>]*><!--\[-->desktop</);
  assert.match(html, /data-state="visible"[^>]*><!--\[-->compact</);
});

test("hydrates with the ssrWidth layout, then adopts the real viewport", async () => {
  window.happyDOM.setViewport({ width: 1300, height: 800 });
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.querySelector('[data-vize-ui="responsive-switch"]')?.textContent, "tablet");
    const states = [...host.querySelectorAll('[data-vize-ui="responsive-show"]')].map((node) =>
      node.getAttribute("data-state"),
    );
    assert.deepEqual(states, ["visible", "hidden"]);
    assert.match(host.textContent ?? "", /desktop/);
    assert.doesNotMatch(host.textContent ?? "", /compact/);
  } finally {
    app.unmount();
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
    window.happyDOM.setViewport({ width: 1024, height: 768 });
  }
  assert.deepEqual(diagnostics, []);
});
