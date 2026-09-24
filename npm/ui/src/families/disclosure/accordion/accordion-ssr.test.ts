import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import AccordionContent from "./accordion-content.vue";
import AccordionHeader from "./accordion-header.vue";
import AccordionItem from "./accordion-item.vue";
import AccordionRoot from "./accordion-root.vue";
import AccordionTrigger from "./accordion-trigger.vue";

function createProbe(props: Record<string, unknown>) {
  return defineComponent({
    name: "AccordionSsrProbe",
    setup() {
      return () =>
        h(AccordionRoot, { type: "single", ...props }, () =>
          ["alpha", "bravo"].map((value) =>
            h(AccordionItem, { key: value, value }, () => [
              h(AccordionHeader, null, () => h(AccordionTrigger, null, () => value)),
              h(AccordionContent, null, () => `${value} details`),
            ]),
          ),
        );
    },
  });
}

test("renders byte-identical accordion markup across isolated SSR requests", async () => {
  const Probe = createProbe({ defaultValue: "alpha" });
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div id="vize-v-\d+-accordion"/);
  assert.match(html, /data-vize-ui="accordion-root"/);
  assert.match(html, /<h3[^>]*data-vize-ui="accordion-header"/);
  assert.match(html, /id="vize-v-\d+-accordion-item-alpha-trigger"/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /aria-disabled="true"/);
  assert.match(html, /aria-controls="vize-v-\d+-accordion-item-bravo-content"/);
  assert.match(html, /id="vize-v-\d+-accordion-item-bravo-content"[^>]*hidden/);
  assert.doesNotMatch(html, /--vize-accordion-content-height/);
});

test("hydrates without mismatches and upgrades closed panels to hidden until-found", async () => {
  const Probe = createProbe({ id: "faq", hiddenUntilFound: true });
  const serverHtml = await renderToString(createSSRApp(Probe));
  assert.doesNotMatch(serverHtml, /hidden="until-found"/);
  assert.match(serverHtml, /data-hidden-until-found="true"/);
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTrigger = host.querySelector("#faq-item-alpha-trigger");
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
    assert.ok(host.querySelector("#faq-item-alpha-trigger") === serverTrigger);
    assert.equal(
      host.querySelector("#faq-item-alpha-content")?.getAttribute("hidden"),
      "until-found",
    );
    assert.deepEqual(diagnostics, []);

    const trigger = host.querySelector<HTMLButtonElement>("#faq-item-bravo-trigger");
    trigger?.click();
    await nextTick();
    assert.equal(host.querySelector("#faq-item-bravo-content")?.hasAttribute("hidden"), false);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
